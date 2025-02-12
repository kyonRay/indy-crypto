#[macro_use]
pub mod logger;
mod commitment;
mod constants;
#[macro_use]
mod datastructures;
#[macro_use]
mod helpers;
mod hash;
pub mod issuer;
pub mod prover;
pub mod verifier;

use bn::BigNumber;
use errors::IndyCryptoError;

use std::collections::{HashMap, HashSet, BTreeSet, BTreeMap};
use std::hash::Hash;

/// Creates random nonce
///
/// # Example
/// ```
/// use indy_crypto::cl::new_nonce;
///
/// let _nonce = new_nonce().unwrap();
/// ```
pub fn new_nonce() -> Result<Nonce, IndyCryptoError> {
    Ok(helpers::bn_rand(constants::LARGE_NONCE)?)
}

/// A list of attributes a Credential is based on.
#[derive(Debug, Clone)]
pub struct CredentialSchema {
    attrs: BTreeSet<String>, /* attr names */
}

/// A Builder of `Credential Schema`.
#[derive(Debug)]
pub struct CredentialSchemaBuilder {
    attrs: BTreeSet<String>, /* attr names */
}

impl CredentialSchemaBuilder {
    pub fn new() -> Result<CredentialSchemaBuilder, IndyCryptoError> {
        Ok(CredentialSchemaBuilder { attrs: BTreeSet::new() })
    }

    pub fn add_attr(&mut self, attr: &str) -> Result<(), IndyCryptoError> {
        self.attrs.insert(attr.to_owned());
        Ok(())
    }

    pub fn finalize(self) -> Result<CredentialSchema, IndyCryptoError> {
        Ok(CredentialSchema { attrs: self.attrs })
    }
}

#[derive(Debug, Clone)]
pub struct NonCredentialSchema {
    attrs: BTreeSet<String>,
}

#[derive(Debug)]
pub struct NonCredentialSchemaBuilder {
    attrs: BTreeSet<String>,
}

impl NonCredentialSchemaBuilder {
    pub fn new() -> Result<NonCredentialSchemaBuilder, IndyCryptoError> {
        Ok(NonCredentialSchemaBuilder {
            attrs: BTreeSet::new(),
        })
    }

    pub fn add_attr(&mut self, attr: &str) -> Result<(), IndyCryptoError> {
        self.attrs.insert(attr.to_owned());
        Ok(())
    }

    pub fn finalize(self) -> Result<NonCredentialSchema, IndyCryptoError> {
        Ok(NonCredentialSchema { attrs: self.attrs })
    }
}

/// The m value for attributes,
/// commitments also store a blinding factor
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum CredentialValue {
    Known { value: BigNumber }, //Issuer and Prover know these
    Hidden { value: BigNumber }, //Only known to Prover who binds these into the U factor
    Commitment {
        value: BigNumber,
        blinding_factor: BigNumber,
    }, //Only known to Prover, not included in the credential, used for proving knowledge during issuance
}

impl CredentialValue {
    pub fn clone(&self) -> Result<CredentialValue, IndyCryptoError> {
        Ok(match *self {
            CredentialValue::Known { ref value } => CredentialValue::Known {
                value: value.clone()?,
            },
            CredentialValue::Hidden { ref value } => CredentialValue::Hidden {
                value: value.clone()?,
            },
            CredentialValue::Commitment {
                ref value,
                ref blinding_factor,
            } => CredentialValue::Commitment {
                value: value.clone()?,
                blinding_factor: blinding_factor.clone()?,
            },
        })
    }

    pub fn is_known(&self) -> bool {
        match *self {
            CredentialValue::Known { .. } => true,
            _ => false,
        }
    }

    pub fn is_hidden(&self) -> bool {
        match *self {
            CredentialValue::Hidden { .. } => true,
            _ => false,
        }
    }

    pub fn is_commitment(&self) -> bool {
        match *self {
            CredentialValue::Commitment { .. } => true,
            _ => false,
        }
    }

    pub fn value(&self) -> &BigNumber {
        match *self {
            CredentialValue::Known { ref value } => value,
            CredentialValue::Hidden { ref value } => value,
            CredentialValue::Commitment { ref value, .. } => value,
        }
    }
}

/// Values of attributes from `Claim Schema` (must be integers).
#[derive(Debug)]
pub struct CredentialValues {
    attrs_values: BTreeMap<String, CredentialValue>,
}

impl CredentialValues {
    pub fn clone(&self) -> Result<CredentialValues, IndyCryptoError> {
        Ok(CredentialValues {
            attrs_values: clone_credential_value_map(&self.attrs_values)?
        })
    }
}

/// A Builder of `Credential Values`.
#[derive(Debug)]
pub struct CredentialValuesBuilder {
    attrs_values: BTreeMap<String, CredentialValue>, /* attr_name -> int representation of value */
}

impl CredentialValuesBuilder {
    pub fn new() -> Result<CredentialValuesBuilder, IndyCryptoError> {
        Ok(CredentialValuesBuilder { attrs_values: BTreeMap::new() })
    }

    pub fn add_dec_known(&mut self, attr: &str, value: &str) -> Result<(), IndyCryptoError> {
        self.attrs_values.insert(
            attr.to_owned(),
            CredentialValue::Known { value: BigNumber::from_dec(value)? },
        );
        Ok(())
    }

    pub fn add_dec_hidden(&mut self, attr: &str, value: &str) -> Result<(), IndyCryptoError> {
        self.attrs_values.insert(
            attr.to_owned(),
            CredentialValue::Hidden { value: BigNumber::from_dec(value)? },
        );
        Ok(())
    }

    pub fn add_dec_commitment(
        &mut self,
        attr: &str,
        value: &str,
        blinding_factor: &str,
    ) -> Result<(), IndyCryptoError> {
        self.attrs_values.insert(
            attr.to_owned(),
            CredentialValue::Commitment {
                value: BigNumber::from_dec(value)?,
                blinding_factor: BigNumber::from_dec(blinding_factor)?,
            },
        );
        Ok(())
    }

    pub fn add_value_known(
        &mut self,
        attr: &str,
        value: &BigNumber,
    ) -> Result<(), IndyCryptoError> {
        self.attrs_values.insert(
            attr.to_owned(),
            CredentialValue::Known { value: value.clone()? },
        );
        Ok(())
    }

    pub fn add_value_hidden(
        &mut self,
        attr: &str,
        value: &BigNumber,
    ) -> Result<(), IndyCryptoError> {
        self.attrs_values.insert(
            attr.to_owned(),
            CredentialValue::Hidden { value: value.clone()? },
        );
        Ok(())
    }

    pub fn add_value_commitment(
        &mut self,
        attr: &str,
        value: &BigNumber,
        blinding_factor: &BigNumber,
    ) -> Result<(), IndyCryptoError> {
        self.attrs_values.insert(
            attr.to_owned(),
            CredentialValue::Commitment {
                value: value.clone()?,
                blinding_factor: blinding_factor.clone()?,
            },
        );
        Ok(())
    }

    pub fn finalize(self) -> Result<CredentialValues, IndyCryptoError> {
        Ok(CredentialValues { attrs_values: self.attrs_values })
    }
}

/// `Issuer Public Key` contains 2 internal parts.
/// One for signing primary credentials and second for signing non-revocation credentials.
/// These keys are used to proof that credential was issued and doesn’t revoked by this issuer.
/// Issuer keys have global identifier that must be known to all parties.
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct CredentialPublicKey {
    p_key: CredentialPrimaryPublicKey
}

impl CredentialPublicKey {
    pub fn clone(&self) -> Result<CredentialPublicKey, IndyCryptoError> {
        Ok(CredentialPublicKey {
            p_key: self.p_key.clone()?
        })
    }

    pub fn get_primary_key(&self) -> Result<CredentialPrimaryPublicKey, IndyCryptoError> {
        Ok(self.p_key.clone()?)
    }

    pub fn build_from_parts(p_key: &CredentialPrimaryPublicKey) -> Result<CredentialPublicKey, IndyCryptoError> {
        Ok(CredentialPublicKey {
            p_key: p_key.clone()?
        })
    }
}

/// `Issuer Private Key`: contains 2 internal parts.
/// One for signing primary credentials and second for signing non-revocation credentials.
#[derive(Debug, Deserialize, Serialize)]
pub struct CredentialPrivateKey {
    p_key: CredentialPrimaryPrivateKey
}

/// Issuer's "Public Key" is used to verify the Issuer's signature over the Credential's attributes' values (primary credential).
#[derive(Debug, PartialEq, Serialize)]
pub struct CredentialPrimaryPublicKey {
    n: BigNumber,
    s: BigNumber,
    r: HashMap<String /* attr_name */, BigNumber>,
    z: BigNumber
}

impl CredentialPrimaryPublicKey {
    pub fn clone(&self) -> Result<CredentialPrimaryPublicKey, IndyCryptoError> {
        Ok(CredentialPrimaryPublicKey {
            n: self.n.clone()?,
            s: self.s.clone()?,
            r: clone_bignum_map(&self.r)?,
            z: self.z.clone()?
        })
    }
}

impl <'a> ::serde::de::Deserialize<'a> for CredentialPrimaryPublicKey {
    fn deserialize<D: ::serde::de::Deserializer<'a>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct CredentialPrimaryPublicKeyV1 {
            n: BigNumber,
            s: BigNumber,
            r: HashMap<String /* attr_name */, BigNumber>,
            #[serde(default)]
            rms: BigNumber,
            z: BigNumber
        }

        let mut helper = CredentialPrimaryPublicKeyV1::deserialize(deserializer)?;
        if helper.rms != BigNumber::default() {
            helper.r.insert("master_secret".to_string(), helper.rms);
        }
        Ok(CredentialPrimaryPublicKey {
            n: helper.n,
            s: helper.s,
            z: helper.z,
            r: helper.r
        })
    }
}

/// Issuer's "Private Key" used for signing Credential's attributes' values (primary credential)
#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct CredentialPrimaryPrivateKey {
    p: BigNumber,
    q: BigNumber
}

/// `Primary Public Key Metadata` required for building of Proof Correctness of `Issuer Public Key`
#[derive(Debug)]
pub struct CredentialPrimaryPublicKeyMetadata {
    xz: BigNumber,
    xr: HashMap<String, BigNumber>
}

/// Proof of `Issuer Public Key` correctness
#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct CredentialKeyCorrectnessProof {
    c: BigNumber,
    xz_cap: BigNumber,
    xr_cap: Vec<(String, BigNumber)>,
}

/// Issuer's signature over Credential attribute values.
#[derive(Debug, Deserialize, Serialize)]
pub struct CredentialSignature {
    p_credential: PrimaryCredentialSignature,
}

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct PrimaryCredentialSignature {
    a: BigNumber,
    e: BigNumber,
    v: BigNumber
}

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct SignatureCorrectnessProof {
    se: BigNumber,
    c: BigNumber
}

/// Secret key encoded in a credential that is used to prove that prover owns the credential; can be used to
/// prove linkage across credentials.
/// Prover blinds master secret, generating `BlindedCredentialSecrets` and `CredentialSecretsBlindingFactors` (blinding factors)
/// and sends the `BlindedCredentialSecrets` to Issuer who then encodes it credential creation.
/// The blinding factors are used by Prover for post processing of issued credentials.
#[derive(Debug, Deserialize, Serialize)]
pub struct MasterSecret {
    ms: BigNumber,
}

impl MasterSecret {
    pub fn clone(&self) -> Result<MasterSecret, IndyCryptoError> {
        Ok(MasterSecret { ms: self.ms.clone()? })
    }

    pub fn value(&self) -> Result<BigNumber, IndyCryptoError> {
        Ok(self.ms.clone()?)
    }
}

/// Blinded Master Secret uses by Issuer in credential creation.
#[derive(Debug, Deserialize, Serialize)]
pub struct BlindedCredentialSecrets {
    u: BigNumber,
    hidden_attributes: BTreeSet<String>,
    committed_attributes: BTreeMap<String, BigNumber>
}

/// `CredentialSecretsBlindingFactors` used by Prover for post processing of credentials received from Issuer.
#[derive(Debug, Deserialize, Serialize)]
pub struct CredentialSecretsBlindingFactors {
    v_prime: BigNumber
}

#[derive(Eq, PartialEq, Debug)]
pub struct PrimaryBlindedCredentialSecretsFactors {
    u: BigNumber,
    v_prime: BigNumber,
    hidden_attributes: BTreeSet<String>,
    committed_attributes: BTreeMap<String, BigNumber>,
}

#[derive(Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct BlindedCredentialSecretsCorrectnessProof {
    c: BigNumber, // Fiat-Shamir challenge hash
    v_dash_cap: BigNumber, // Value to prove knowledge of `u` construction in `BlindedCredentialSecrets`
    m_caps: BTreeMap<String, BigNumber>, // Values for proving knowledge of committed values
    r_caps: BTreeMap<String, BigNumber>, // Blinding values for m_caps
}

/// “Sub Proof Request” - input to create a Proof for a credential;
/// Contains attributes to be revealed and predicates.
#[derive(Debug, Clone)]
pub struct SubProofRequest {
    revealed_attrs: BTreeSet<String>,
    predicates: BTreeSet<Predicate>,
}

/// Builder of “Sub Proof Request”.
#[derive(Debug)]
pub struct SubProofRequestBuilder {
    value: SubProofRequest
}

impl SubProofRequestBuilder {
    pub fn new() -> Result<SubProofRequestBuilder, IndyCryptoError> {
        Ok(SubProofRequestBuilder {
            value: SubProofRequest {
                revealed_attrs: BTreeSet::new(),
                predicates: BTreeSet::new()
            }
        })
    }

    pub fn add_revealed_attr(&mut self, attr: &str) -> Result<(), IndyCryptoError> {
        self.value.revealed_attrs.insert(attr.to_owned());
        Ok(())
    }

    pub fn add_predicate(&mut self, attr_name: &str, p_type: &str, value: i32) -> Result<(), IndyCryptoError> {
        let p_type = match p_type {
            "GE" => PredicateType::GE,
            "LE" => PredicateType::LE,
            "GT" => PredicateType::GT,
            "LT" => PredicateType::LT,
            p_type => return Err(IndyCryptoError::InvalidStructure(format!("Invalid predicate type: {:?}", p_type)))
        };

        let predicate = Predicate {
            attr_name: attr_name.to_owned(),
            p_type,
            value
        };

        self.value.predicates.insert(predicate);
        Ok(())
    }

    pub fn finalize(self) -> Result<SubProofRequest, IndyCryptoError> {
        Ok(self.value)
    }
}

/// Some condition that must be satisfied.
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
pub struct Predicate {
    attr_name: String,
    p_type: PredicateType,
    value: i32,
}

impl Predicate {
    pub fn get_delta(&self, attr_value: i32) -> i32 {
        match self.p_type {
            PredicateType::GE => attr_value - self.value,
            PredicateType::GT => attr_value - self.value - 1,
            PredicateType::LE => self.value - attr_value,
            PredicateType::LT => self.value - attr_value - 1
        }
    }

    pub fn get_delta_prime(&self) -> Result<BigNumber, IndyCryptoError> {
        match self.p_type {
            PredicateType::GE => BigNumber::from_dec(&self.value.to_string()),
            PredicateType::GT => BigNumber::from_dec(&(self.value + 1).to_string()),
            PredicateType::LE => BigNumber::from_dec(&self.value.to_string()),
            PredicateType::LT => BigNumber::from_dec(&(self.value - 1).to_string())
        }
    }

    pub fn is_less(&self) -> bool {
        match self.p_type {
            PredicateType::GE | PredicateType::GT => false,
            PredicateType::LE | PredicateType::LT => true
        }
    }
}

/// Condition type
#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
pub enum PredicateType {
    GE,
    LE,
    GT,
    LT
}

/// Proof is complex crypto structure created by prover over multiple credentials that allows to prove that prover:
/// 1) Knows signature over credentials issued with specific issuer keys (identified by key id)
/// 2) Credential contains attributes with specific values that prover wants to disclose
/// 3) Credential contains attributes with valid predicates that verifier wants the prover to satisfy.
#[derive(Debug, Deserialize, Serialize)]
pub struct Proof {
    proofs: Vec<SubProof>,
    aggregated_proof: AggregatedProof,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SubProof {
    primary_proof: PrimaryProof
}

#[derive(Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct AggregatedProof {
    c_hash: BigNumber,
    c_list: Vec<Vec<u8>>
}

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct PrimaryProof {
    eq_proof: PrimaryEqualProof,
    ne_proofs: Vec<PrimaryPredicateInequalityProof>
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct PrimaryEqualProof {
    revealed_attrs: BTreeMap<String /* attr_name of revealed */, BigNumber>,
    a_prime: BigNumber,
    e: BigNumber,
    v: BigNumber,
    m: HashMap<String /* attr_name of all except revealed */, BigNumber>
}

impl <'a> ::serde::de::Deserialize<'a> for PrimaryEqualProof {
    fn deserialize<D: ::serde::de::Deserializer<'a>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct PrimaryEqualProofV1 {
            revealed_attrs: BTreeMap<String /* attr_name of revealed */, BigNumber>,
            a_prime: BigNumber,
            e: BigNumber,
            v: BigNumber,
            m: HashMap<String /* attr_name of all except revealed */, BigNumber>,
            #[serde(default)]
            m1: BigNumber
        }

        let mut helper = PrimaryEqualProofV1::deserialize(deserializer)?;
        if helper.m1 != BigNumber::default() {
            helper.m.insert("master_secret".to_string(), helper.m1);
        }
        Ok(PrimaryEqualProof {
            revealed_attrs: helper.revealed_attrs,
            a_prime: helper.a_prime,
            e: helper.e,
            v: helper.v,
            m: helper.m
        })
    }
}

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct PrimaryPredicateInequalityProof {
    u: HashMap<String, BigNumber>,
    r: HashMap<String, BigNumber>,
    mj: BigNumber,
    alpha: BigNumber,
    t: HashMap<String, BigNumber>,
    predicate: Predicate
}

#[derive(Debug)]
pub struct InitProof {
    primary_init_proof: PrimaryInitProof,
    credential_values: CredentialValues,
    sub_proof_request: SubProofRequest,
    credential_schema: CredentialSchema,
    non_credential_schema: NonCredentialSchema,
}


#[derive(Debug, Eq, PartialEq)]
pub struct PrimaryInitProof {
    eq_proof: PrimaryEqualInitProof,
    ne_proofs: Vec<PrimaryPredicateInequalityInitProof>
}

impl PrimaryInitProof {
    pub fn as_c_list(&self) -> Result<Vec<Vec<u8>>, IndyCryptoError> {
        let mut c_list: Vec<Vec<u8>> = self.eq_proof.as_list()?;
        for ne_proof in self.ne_proofs.iter() {
            c_list.append_vec(ne_proof.as_list()?)?;
        }
        Ok(c_list)
    }

    pub fn as_tau_list(&self) -> Result<Vec<Vec<u8>>, IndyCryptoError> {
        let mut tau_list: Vec<Vec<u8>> = self.eq_proof.as_tau_list()?;
        for ne_proof in self.ne_proofs.iter() {
            tau_list.append_vec(ne_proof.as_tau_list()?)?;
        }
        Ok(tau_list)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct PrimaryEqualInitProof {
    a_prime: BigNumber,
    t: BigNumber,
    e_tilde: BigNumber,
    e_prime: BigNumber,
    v_tilde: BigNumber,
    v_prime: BigNumber,
    m_tilde: HashMap<String, BigNumber>
}

impl PrimaryEqualInitProof {
    pub fn as_list(&self) -> Result<Vec<Vec<u8>>, IndyCryptoError> {
        Ok(vec![self.a_prime.to_bytes()?])
    }

    pub fn as_tau_list(&self) -> Result<Vec<Vec<u8>>, IndyCryptoError> {
        Ok(vec![self.t.to_bytes()?])
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct PrimaryPredicateInequalityInitProof {
    c_list: Vec<BigNumber>,
    tau_list: Vec<BigNumber>,
    u: HashMap<String, BigNumber>,
    u_tilde: HashMap<String, BigNumber>,
    r: HashMap<String, BigNumber>,
    r_tilde: HashMap<String, BigNumber>,
    alpha_tilde: BigNumber,
    predicate: Predicate,
    t: HashMap<String, BigNumber>,
}

impl PrimaryPredicateInequalityInitProof {
    pub fn as_list(&self) -> Result<&Vec<BigNumber>, IndyCryptoError> {
        Ok(&self.c_list)
    }

    pub fn as_tau_list(&self) -> Result<&Vec<BigNumber>, IndyCryptoError> {
        Ok(&self.tau_list)
    }
}

/// Random BigNumber that uses `Prover` for proof generation and `Verifier` for proof verification.
pub type Nonce = BigNumber;

#[derive(Debug)]
pub struct VerifiableCredential {
    pub_key: CredentialPublicKey,
    sub_proof_request: SubProofRequest,
    credential_schema: CredentialSchema,
    non_credential_schema: NonCredentialSchema
}

trait BytesView {
    fn to_bytes(&self) -> Result<Vec<u8>, IndyCryptoError>;
}

impl BytesView for BigNumber {
    fn to_bytes(&self) -> Result<Vec<u8>, IndyCryptoError> {
        Ok(self.to_bytes()?)
    }
}

trait AppendByteArray {
    fn append_vec<T: BytesView>(&mut self, other: &Vec<T>) -> Result<(), IndyCryptoError>;
}

impl AppendByteArray for Vec<Vec<u8>> {
    fn append_vec<T: BytesView>(&mut self, other: &Vec<T>) -> Result<(), IndyCryptoError> {
        for el in other.iter() {
            self.push(el.to_bytes()?);
        }
        Ok(())
    }
}

fn clone_bignum_map<K: Clone + Eq + Hash>(other: &HashMap<K, BigNumber>) -> Result<HashMap<K, BigNumber>, IndyCryptoError> {
    let mut res = HashMap::new();
    for (k, v) in other.iter() {
        res.insert(k.clone(), v.clone()?);
    }
    Ok(res)
}


fn clone_credential_value_map<K: Clone + Eq + Ord>(other: &BTreeMap<K, CredentialValue>) -> Result<BTreeMap<K, CredentialValue>, IndyCryptoError> {
    let mut res = BTreeMap::new();
    for (k, v) in other {
        res.insert(k.clone(), v.clone()?);
    }
    Ok(res)
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_json;
    use self::issuer::Issuer;
    use self::prover::Prover;
    use self::verifier::Verifier;
    
    #[test]
    fn multiple_predicates() {
        let mut credential_schema_builder = Issuer::new_credential_schema_builder().unwrap();
        credential_schema_builder.add_attr("funds_sold_and_securities_purchased").unwrap();
        credential_schema_builder.add_attr("other_earning_assets").unwrap();
        credential_schema_builder.add_attr("cash").unwrap();
        credential_schema_builder.add_attr("allowance").unwrap();
        credential_schema_builder.add_attr("total_assets").unwrap();
        credential_schema_builder.add_attr("domestic_interest_bearing_deposits").unwrap();
        credential_schema_builder.add_attr("funds_purchased").unwrap();
        credential_schema_builder.add_attr("long_term_debt").unwrap();
        credential_schema_builder.add_attr("non_interest_bearing_liabilities").unwrap();
        credential_schema_builder.add_attr("shareholder_equity").unwrap();
        credential_schema_builder.add_attr("total_liabilities").unwrap();

        let credential_schema = credential_schema_builder.finalize().unwrap();

        let mut non_credential_schema_builder = NonCredentialSchemaBuilder::new().unwrap();
        non_credential_schema_builder.add_attr("master_secret").unwrap();
        let non_credential_schema = non_credential_schema_builder.finalize().unwrap();

        let (cred_pub_key, cred_priv_key, cred_key_correctness_proof) = Issuer::new_credential_def(&credential_schema, &non_credential_schema).unwrap();

        let master_secret = Prover::new_master_secret().unwrap();
        let credential_nonce = new_nonce().unwrap();

        let mut credential_values_builder = Issuer::new_credential_values_builder().unwrap();
        credential_values_builder.add_value_hidden("master_secret", &master_secret.value().unwrap()).unwrap();
        credential_values_builder.add_dec_known("funds_sold_and_securities_purchased", "50").unwrap();
        credential_values_builder.add_dec_known("other_earning_assets", "60").unwrap();
        credential_values_builder.add_dec_known("cash", "70").unwrap();
        credential_values_builder.add_dec_known("allowance", "80").unwrap();
        credential_values_builder.add_dec_known("total_assets", "260").unwrap();

        credential_values_builder.add_dec_known("domestic_interest_bearing_deposits", "10").unwrap();
        credential_values_builder.add_dec_known("funds_purchased", "20").unwrap();
        credential_values_builder.add_dec_known("long_term_debt", "30").unwrap();
        credential_values_builder.add_dec_known("non_interest_bearing_liabilities", "40").unwrap();
        credential_values_builder.add_dec_known("shareholder_equity", "50").unwrap();
        credential_values_builder.add_dec_known("total_liabilities", "150").unwrap();
        let cred_values = credential_values_builder.finalize().unwrap();

        let (blinded_credential_secrets, credential_secrets_blinding_factors, blinded_credential_secrets_correctness_proof) =
            Prover::blind_credential_secrets(&cred_pub_key,
                                        &cred_key_correctness_proof,
                                        &cred_values,
                                        &credential_nonce).unwrap();

        let cred_issuance_nonce = new_nonce().unwrap();

        let (mut cred_signature, signature_correctness_proof) = Issuer::sign_credential("b977afe22b5b446109797ad925d9f133fc33c1914081071295d2ac1ddce3385d",
                                                                                        &blinded_credential_secrets,
                                                                                        &blinded_credential_secrets_correctness_proof,
                                                                                        &credential_nonce,
                                                                                        &cred_issuance_nonce,
                                                                                        &cred_values,
                                                                                        &cred_pub_key,
                                                                                        &cred_priv_key).unwrap();

        Prover::process_credential_signature(&mut cred_signature,
                                             &cred_values,
                                             &signature_correctness_proof,
                                             &credential_secrets_blinding_factors,
                                             &cred_pub_key,
                                             &cred_issuance_nonce).unwrap();

        let mut sub_proof_request_builder = Verifier::new_sub_proof_request_builder().unwrap();
        sub_proof_request_builder.add_revealed_attr("total_liabilities").unwrap();

        sub_proof_request_builder.add_predicate("funds_sold_and_securities_purchased", "LT", 100).unwrap();
        sub_proof_request_builder.add_predicate("funds_sold_and_securities_purchased", "GT", 0).unwrap();
        sub_proof_request_builder.add_predicate("other_earning_assets", "LT", 100).unwrap();
        sub_proof_request_builder.add_predicate("cash", "LT", 100).unwrap();
        sub_proof_request_builder.add_predicate("allowance", "LT", 100).unwrap();
        sub_proof_request_builder.add_predicate("total_assets", "GT", 100).unwrap();

        sub_proof_request_builder.add_predicate("domestic_interest_bearing_deposits", "LE", 100).unwrap();
        sub_proof_request_builder.add_predicate("funds_purchased", "LE", 100).unwrap();
        sub_proof_request_builder.add_predicate("long_term_debt", "LE", 100).unwrap();
        sub_proof_request_builder.add_predicate("non_interest_bearing_liabilities", "LE", 100).unwrap();
        sub_proof_request_builder.add_predicate("shareholder_equity", "LE", 100).unwrap();
        let sub_proof_request = sub_proof_request_builder.finalize().unwrap();

        let mut proof_builder = Prover::new_proof_builder().unwrap();
        proof_builder.add_common_attribute("master_secret").unwrap();
        proof_builder.add_sub_proof_request(&sub_proof_request,
                                            &credential_schema,
                                            &non_credential_schema,
                                            &cred_signature,
                                            &cred_values,
                                            &cred_pub_key).unwrap();

        let proof_request_nonce = new_nonce().unwrap();
        let proof = proof_builder.finalize(&proof_request_nonce).unwrap();

        let mut proof_verifier = Verifier::new_proof_verifier().unwrap();
        proof_verifier.add_sub_proof_request(&sub_proof_request,
                                             &credential_schema,
                                             &non_credential_schema,
                                             &cred_pub_key).unwrap();
        assert!(proof_verifier.verify(&proof, &proof_request_nonce).unwrap());
    }

    #[test]
    fn credential_primary_public_key_conversion_works() {
        let string1 = r#"{
                 "n":"94752773003676215520340390286428145970577435379747248974837494389412082076547661891067434652276048522392442077335235388384984508621151996372559370276527598415204914831299768834758349425880859567795461321350412568232531440683627330032285846734752711268206613305069973750567165548816744023441650243801226580089078611213688037852063937259593837571943085718154394160122127891902723469618952030300431400181642597638732611518885616750614674142486169255034160093153314427704384760404032620300207070597238445621198019686315730573836193179483581719638565112589368474184957790046080767607443902003396643479910885086397579016949",
                 "s":"69412039600361800795429063472749802282903100455399422661844374992112119187258494682747330126416608111152308407310993289705267392969490079422545377823004584691698371089275086755756916575365439635768831063415050875440259347714303092581127338698890829662982679857654396534761554232914231213603075653629534596880597317047082696083166437821687405393805812336036647064899914817619861844092002636340952247588092904075021313598848481976631171767602864723880294787434756140969093416957086578979859382777377267118038126527549503876861370823520292585383483415337137062969402135540724590433024573312636828352734474276871187481042",
                 "r":{
                    "age":"90213462228557102785520674066817329607065098280886260103565465379328385444439123494955469500769864345819799623656302322427095342533906338563811194606234218499052997878891037890681314502037670093285650999142741875494918117023196753133733183769000368858655309319559871473827485381905587653145346258174022279515774231018893119774525087260785417971477049379955435611260162822960318458092151247522911151421981946748062572207451174079699745404644326303405628719711440096340436702151418321760375229323874027809433387030362543124015034968644213166988773750220839778654632868402703075643503247560457217265822566406481434257658",
                    "height":"5391629214047043372090966654120333203094518833743674393685635640778311836867622750170495792524304436281896432811455146477306501487333852472234525296058562723428516533641819658096275918819548576029252844651857904411902677509566190811985500618327955392620642519618001469964706236997279744030829811760566269297728600224591162795849338756438466021999870256717098048301453122263380103723520670896747657149140787953289875480355961166269553534983692005983375091110745903845958291035125718192228291126861666488320123420563113398593180368102996188897121307947248313167444374640621348136184583596487812048321382789134349482978",
                    "name":"77620276231641170120118188540269028385259155493880444038204934044861538875241492581309232702380290690573764595644801264135299029620031922004969464948925209245961139274806949465303313280327009910224580146266877846633558282936147503639084871235301887617650455108586169172459479774206351621894071684884758716731250212971549835402948093455393537573942251389197338609379019568250835525301455105289583537704528678164781839386485243301381405947043141406604458853106372019953011725448481499511842635580639867624862131749700424467221215201558826025502015289693451254344465767556321748122037274143231500322140291667454975911415",
                    "sex":"9589127953934298285127566793382980040568251918610023890115614786922171891298122457059996745443282235104668609426602496632245081143706804923757991602521162900045665258654877250328921570207935035808607238170708932487500434929591458680514420504595293934408583558084774019418964434729989362874165849497341625769388145344718883550286508846516335790153998186614300493752317413537864956171451048868305380731285315760405126912629495204641829764230906698870575251861738847175174907714361155400020318026100833368698707674675548636610079631382774152211885405135045997623813094890524761824654025566099289284433567918244183562578"
                 },
                 "rms": "51663676247842478814965591806476166314018329779100758392678204435864101706276421100107118776199283981546682625125866769910726045178868995629346547166162207336629797340989495021248125384357605197654315399409367101440127312902706857104045262430326903112478154165057770802221835566137181123204394005042244715693211063132775814710986488082414421678086296488865286754803461178476006057306298883090062534704773627985221339716152111236985859907502262026150818487846053415153813804554830872575193396851274528558072704096323791923604931528594861707067370303707070124331485728734993074005001622035563911923643592706985074084035",
                 "rctxt":"60293229766149238310917923493206871325969738638348535857162249827595080348039120693847207728852550647187915587987334466582959087190830489258423645708276339586344792464665557038628519694583193692804909304334143467285824750999826903922956158114736424517794036832742439893595716442609416914557200249087236453529632524328334442017327755310827841619727229956823928475210644630763245343116656886668444813463622336899670813312626960927341115875144198394937398391514458462051400588820774593570752884252721428948286332429715774158007033348855655388287735570407811513582431434394169600082273657382209764160600063473877124656503",
                 "z":"70486542646006986754234343446999146345523665952265004264483059055307042644604796098478326629348068818272043688144751523020343994424262034067120716287162029288580118176972850899641747743901392814182335879624697285262287085187745166728443417803755667806532945136078671895589773743252882095592683767377435647759252676700424432160196120135306640079450582642553870190550840243254909737360996391470076977433525925799327058405911708739601511578904084479784054523375804238021939950198346585735956776232824298799161587408330541161160988641895300133750453032202142977745163418534140360029475702333980267724847703258887949227842"
              }"#;

        let string2 = r#"{
                 "n":"94752773003676215520340390286428145970577435379747248974837494389412082076547661891067434652276048522392442077335235388384984508621151996372559370276527598415204914831299768834758349425880859567795461321350412568232531440683627330032285846734752711268206613305069973750567165548816744023441650243801226580089078611213688037852063937259593837571943085718154394160122127891902723469618952030300431400181642597638732611518885616750614674142486169255034160093153314427704384760404032620300207070597238445621198019686315730573836193179483581719638565112589368474184957790046080767607443902003396643479910885086397579016949",
                 "s":"69412039600361800795429063472749802282903100455399422661844374992112119187258494682747330126416608111152308407310993289705267392969490079422545377823004584691698371089275086755756916575365439635768831063415050875440259347714303092581127338698890829662982679857654396534761554232914231213603075653629534596880597317047082696083166437821687405393805812336036647064899914817619861844092002636340952247588092904075021313598848481976631171767602864723880294787434756140969093416957086578979859382777377267118038126527549503876861370823520292585383483415337137062969402135540724590433024573312636828352734474276871187481042",
                 "r":{
                    "age":"90213462228557102785520674066817329607065098280886260103565465379328385444439123494955469500769864345819799623656302322427095342533906338563811194606234218499052997878891037890681314502037670093285650999142741875494918117023196753133733183769000368858655309319559871473827485381905587653145346258174022279515774231018893119774525087260785417971477049379955435611260162822960318458092151247522911151421981946748062572207451174079699745404644326303405628719711440096340436702151418321760375229323874027809433387030362543124015034968644213166988773750220839778654632868402703075643503247560457217265822566406481434257658",
                    "height":"5391629214047043372090966654120333203094518833743674393685635640778311836867622750170495792524304436281896432811455146477306501487333852472234525296058562723428516533641819658096275918819548576029252844651857904411902677509566190811985500618327955392620642519618001469964706236997279744030829811760566269297728600224591162795849338756438466021999870256717098048301453122263380103723520670896747657149140787953289875480355961166269553534983692005983375091110745903845958291035125718192228291126861666488320123420563113398593180368102996188897121307947248313167444374640621348136184583596487812048321382789134349482978",
                    "name":"77620276231641170120118188540269028385259155493880444038204934044861538875241492581309232702380290690573764595644801264135299029620031922004969464948925209245961139274806949465303313280327009910224580146266877846633558282936147503639084871235301887617650455108586169172459479774206351621894071684884758716731250212971549835402948093455393537573942251389197338609379019568250835525301455105289583537704528678164781839386485243301381405947043141406604458853106372019953011725448481499511842635580639867624862131749700424467221215201558826025502015289693451254344465767556321748122037274143231500322140291667454975911415",
                    "sex":"9589127953934298285127566793382980040568251918610023890115614786922171891298122457059996745443282235104668609426602496632245081143706804923757991602521162900045665258654877250328921570207935035808607238170708932487500434929591458680514420504595293934408583558084774019418964434729989362874165849497341625769388145344718883550286508846516335790153998186614300493752317413537864956171451048868305380731285315760405126912629495204641829764230906698870575251861738847175174907714361155400020318026100833368698707674675548636610079631382774152211885405135045997623813094890524761824654025566099289284433567918244183562578",
                    "master_secret": "51663676247842478814965591806476166314018329779100758392678204435864101706276421100107118776199283981546682625125866769910726045178868995629346547166162207336629797340989495021248125384357605197654315399409367101440127312902706857104045262430326903112478154165057770802221835566137181123204394005042244715693211063132775814710986488082414421678086296488865286754803461178476006057306298883090062534704773627985221339716152111236985859907502262026150818487846053415153813804554830872575193396851274528558072704096323791923604931528594861707067370303707070124331485728734993074005001622035563911923643592706985074084035"
                 },
                 "rctxt":"60293229766149238310917923493206871325969738638348535857162249827595080348039120693847207728852550647187915587987334466582959087190830489258423645708276339586344792464665557038628519694583193692804909304334143467285824750999826903922956158114736424517794036832742439893595716442609416914557200249087236453529632524328334442017327755310827841619727229956823928475210644630763245343116656886668444813463622336899670813312626960927341115875144198394937398391514458462051400588820774593570752884252721428948286332429715774158007033348855655388287735570407811513582431434394169600082273657382209764160600063473877124656503",
                 "z":"70486542646006986754234343446999146345523665952265004264483059055307042644604796098478326629348068818272043688144751523020343994424262034067120716287162029288580118176972850899641747743901392814182335879624697285262287085187745166728443417803755667806532945136078671895589773743252882095592683767377435647759252676700424432160196120135306640079450582642553870190550840243254909737360996391470076977433525925799327058405911708739601511578904084479784054523375804238021939950198346585735956776232824298799161587408330541161160988641895300133750453032202142977745163418534140360029475702333980267724847703258887949227842"
              }"#;

        let one = serde_json::from_str::<CredentialPrimaryPublicKey>(string1).unwrap();
        let two = serde_json::from_str::<CredentialPrimaryPublicKey>(string2).unwrap();

        assert_eq!(two, one);
    }

    #[test]
    fn primary_equal_proof_conversion_works() {
        let string1 = r#"{
            "revealed_attrs":{ "name":"1139481716457488690172217916278103335" },
            "a_prime":"73051896986344783783621559954466052240337632808477729510525777007534198657123370460809453476237905269777928500034476888078179811369103091702326392092669222868996323974762333077146800752404116534730748685092400106417894776122280960547391515814302192999142386455183675790870578615457141270148590712693325301185445330992767208427208215818892089082206123243055148017865514286222759353929656015594529211154843197464055996993778878163967106658629893439206203941596066380562586058713924055616953462170537040600604826428201808405436865130230174790116739542071871153581967170346076628186863101926791732126528122264782281465094",
            "e":"26894279258848531841414955598838798345606055130059418263879278878511424413654641307014787224496208858379991228288791608261549931755104416",
            "v":"769593829417540943566687651216000708099616242062220026508500847265211856977241087739974159673381844796906987056271685312217722655254322996792650873775611656861273544234724432321045515309211146266498852589181986850053751764534235454974453901933962390148609111520973909072559803423360526975061164422239685006387576029266210201929872373313392190241424322333321394922891207577033519614434276723347140746548441162607411616008633618021962845423830579218345578253882839612570986096830936195064001459565147361336597305783767484298283647710212770870573787603073109857430854719681849489345098539472090186844042540487233617799636327572785715912348265648433678177765454231546725849288046905854444755145184654162149010359429569273734847400697627028832950969890252877892391103230391674009825009176344665382964776819962789472959504523580584494299815960094679820651071251157496967617834816772303813309035759721203718921501821175528106375",
            "m":{
                "age":"1143281854280323408461665818853228702279803847691030529301464848501919856277927436364331044530711281448694432838145799412204154542183613877104383361274202256495017144684827419222",
                "sex":"13123681697669364600723785784083768668401173003182555407713667959884184961072036088391942098105496874381346284841774772987179772727928471347011107103459387881602408580853389973314",
                "height":"5824877563809831190436025794795529331411852203759926644567286594845018041324472260994302109635777382645241758582661313361940262319244084725507113643699421966391425299602530147274"
             },
             "m1":"8583218861046444624186479147396651631579156942204850397797096661516116684243552483174250620744158944865553535495733571632663325011575249979223204777745326895517953843420687756433"
         }"#;
        let string2 = r#"{
            "revealed_attrs":{ "name":"1139481716457488690172217916278103335" },
            "a_prime":"73051896986344783783621559954466052240337632808477729510525777007534198657123370460809453476237905269777928500034476888078179811369103091702326392092669222868996323974762333077146800752404116534730748685092400106417894776122280960547391515814302192999142386455183675790870578615457141270148590712693325301185445330992767208427208215818892089082206123243055148017865514286222759353929656015594529211154843197464055996993778878163967106658629893439206203941596066380562586058713924055616953462170537040600604826428201808405436865130230174790116739542071871153581967170346076628186863101926791732126528122264782281465094",
            "e":"26894279258848531841414955598838798345606055130059418263879278878511424413654641307014787224496208858379991228288791608261549931755104416",
            "v":"769593829417540943566687651216000708099616242062220026508500847265211856977241087739974159673381844796906987056271685312217722655254322996792650873775611656861273544234724432321045515309211146266498852589181986850053751764534235454974453901933962390148609111520973909072559803423360526975061164422239685006387576029266210201929872373313392190241424322333321394922891207577033519614434276723347140746548441162607411616008633618021962845423830579218345578253882839612570986096830936195064001459565147361336597305783767484298283647710212770870573787603073109857430854719681849489345098539472090186844042540487233617799636327572785715912348265648433678177765454231546725849288046905854444755145184654162149010359429569273734847400697627028832950969890252877892391103230391674009825009176344665382964776819962789472959504523580584494299815960094679820651071251157496967617834816772303813309035759721203718921501821175528106375",
            "m":{
                "age":"1143281854280323408461665818853228702279803847691030529301464848501919856277927436364331044530711281448694432838145799412204154542183613877104383361274202256495017144684827419222",
                "sex":"13123681697669364600723785784083768668401173003182555407713667959884184961072036088391942098105496874381346284841774772987179772727928471347011107103459387881602408580853389973314",
                "height":"5824877563809831190436025794795529331411852203759926644567286594845018041324472260994302109635777382645241758582661313361940262319244084725507113643699421966391425299602530147274",
                "master_secret":"8583218861046444624186479147396651631579156942204850397797096661516116684243552483174250620744158944865553535495733571632663325011575249979223204777745326895517953843420687756433"
             }
         }"#;

        let one = serde_json::from_str::<PrimaryEqualProof>(string1).unwrap();
        let two = serde_json::from_str::<PrimaryEqualProof>(string2).unwrap();

        assert_eq!(two, one);
    }


    #[test]
    fn demo() {
        let mut credential_schema_builder = Issuer::new_credential_schema_builder().unwrap();
        credential_schema_builder.add_attr("name").unwrap();
        credential_schema_builder.add_attr("sex").unwrap();
        credential_schema_builder.add_attr("age").unwrap();
        credential_schema_builder.add_attr("height").unwrap();
        let credential_schema = credential_schema_builder.finalize().unwrap();

        let mut non_credential_schema_builder = NonCredentialSchemaBuilder::new().unwrap();
        non_credential_schema_builder.add_attr("master_secret").unwrap();
        let non_credential_schema = non_credential_schema_builder.finalize().unwrap();

        let (cred_pub_key, cred_priv_key, cred_key_correctness_proof) = Issuer::new_credential_def(&credential_schema, &non_credential_schema).unwrap();

        let master_secret = Prover::new_master_secret().unwrap();
        let credential_nonce = new_nonce().unwrap();

        let mut credential_values_builder = Issuer::new_credential_values_builder().unwrap();
        credential_values_builder.add_value_hidden("master_secret", &master_secret.value().unwrap()).unwrap();
        credential_values_builder.add_dec_known("name", "1139481716457488690172217916278103335").unwrap();
        credential_values_builder.add_dec_known("sex", "5944657099558967239210949258394887428692050081607692519917050011144233115103").unwrap();
        credential_values_builder.add_dec_known("age", "28").unwrap();
        credential_values_builder.add_dec_known("height", "175").unwrap();
        let cred_values = credential_values_builder.finalize().unwrap();

        let (blinded_credential_secrets, credential_secrets_blinding_factors, blinded_credential_secrets_correctness_proof) =
            Prover::blind_credential_secrets(&cred_pub_key,
                                        &cred_key_correctness_proof,
                                        &cred_values,
                                        &credential_nonce).unwrap();



        let cred_issuance_nonce = new_nonce().unwrap();

        let (mut cred_signature, signature_correctness_proof) = Issuer::sign_credential("CnEDk9HrMnmiHXEV1WFgbVCRteYnPqsJwrTdcZaNhFVW",
                                                                                        &blinded_credential_secrets,
                                                                                        &blinded_credential_secrets_correctness_proof,
                                                                                        &credential_nonce,
                                                                                        &cred_issuance_nonce,
                                                                                        &cred_values,
                                                                                        &cred_pub_key,
                                                                                        &cred_priv_key).unwrap();

        Prover::process_credential_signature(&mut cred_signature,
                                             &cred_values,
                                             &signature_correctness_proof,
                                             &credential_secrets_blinding_factors,
                                             &cred_pub_key,
                                             &cred_issuance_nonce).unwrap();

        let mut sub_proof_request_builder = Verifier::new_sub_proof_request_builder().unwrap();
        sub_proof_request_builder.add_revealed_attr("name").unwrap();
        sub_proof_request_builder.add_predicate("age", "GE", 18).unwrap();
        let sub_proof_request = sub_proof_request_builder.finalize().unwrap();
        let mut proof_builder = Prover::new_proof_builder().unwrap();
        proof_builder.add_common_attribute("master_secret").unwrap();
        proof_builder.add_sub_proof_request(&sub_proof_request,
                                            &credential_schema,
                                            &non_credential_schema,
                                            &cred_signature,
                                            &cred_values,
                                            &cred_pub_key).unwrap();

        let proof_request_nonce = new_nonce().unwrap();
        let proof = proof_builder.finalize(&proof_request_nonce).unwrap();

        let mut proof_verifier = Verifier::new_proof_verifier().unwrap();
        proof_verifier.add_sub_proof_request(&sub_proof_request,
                                             &credential_schema,
                                             &non_credential_schema,
                                             &cred_pub_key).unwrap();
        assert!(proof_verifier.verify(&proof, &proof_request_nonce).unwrap());
    }
}
