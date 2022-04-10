use std::fs::{read, File};
use std::os::unix::prelude::FileExt;

use anyhow::{bail, Result};
use openssl::asn1::Asn1Time;
use openssl::bn::{BigNum, MsbOption};
use openssl::hash::MessageDigest;
use openssl::pkcs12;
use openssl::pkey::{PKey, PKeyRef, Private};
use openssl::rsa::Rsa;
use openssl::x509;
use openssl::x509::extension;
use openssl::x509::X509;

pub struct RootCA {
    pub cert: X509,
    pub key: PKey<Private>,
}

impl RootCA {
    pub fn from_pem(cert_path: &str, key_path: &str) -> Self {
        let cert = cert_from_pem(cert_path).unwrap();
        let key = key_from_pem(key_path).unwrap();
        RootCA { cert, key }
    }
}

pub fn gen_ca() -> Result<(X509, PKey<Private>)> {
    let rsa = Rsa::generate(2048)?;
    let key_pair = PKey::from_rsa(rsa)?;

    let x509_name = subject_name("US", "CA", "RIV", "Rudy", "test")?;

    let mut cert_builder = X509::builder()?;
    cert_builder.set_version(2)?;
    let serial_number = {
        let mut serial = BigNum::new()?;
        serial.rand(159, MsbOption::MAYBE_ZERO, false)?;
        serial.to_asn1_integer()?
    };
    cert_builder.set_serial_number(&serial_number)?;
    cert_builder.set_subject_name(&x509_name)?;
    cert_builder.set_issuer_name(&x509_name)?;
    cert_builder.set_pubkey(&key_pair)?;

    let days = days(365)?;
    cert_builder.set_not_before(&days.0)?;
    cert_builder.set_not_after(&days.1)?;

    cert_builder.append_extension(extension::BasicConstraints::new().critical().ca().build()?)?;
    cert_builder.append_extension(
        extension::KeyUsage::new()
            .critical()
            .key_cert_sign()
            .crl_sign()
            .build()?,
    )?;

    let subject_key_identifier =
        extension::SubjectKeyIdentifier::new().build(&cert_builder.x509v3_context(None, None))?;
    cert_builder.append_extension(subject_key_identifier)?;

    cert_builder.sign(&key_pair, MessageDigest::sha256())?;
    let cert = cert_builder.build();

    Ok((cert, key_pair))
}

fn gen_csr(key_pair: &PKey<Private>) -> Result<x509::X509Req> {
    let mut req_builder = x509::X509ReqBuilder::new()?;
    req_builder.set_pubkey(key_pair)?;

    let x509_name = subject_name("US", "CA", "RIV", "Rudy", "test")?;
    req_builder.set_subject_name(&x509_name)?;

    req_builder.sign(key_pair, MessageDigest::sha256())?;
    let req = req_builder.build();
    Ok(req)
}

pub fn gen_cert(
    ca_cert: &x509::X509Ref,
    ca_key_pair: &PKeyRef<Private>,
    dns_names: Vec<String>,
) -> Result<(X509, PKey<Private>)> {
    let rsa = Rsa::generate(2048)?;
    let key_pair = PKey::from_rsa(rsa)?;

    let req = gen_csr(&key_pair)?;

    let mut cert_builder = X509::builder()?;
    cert_builder.set_version(2)?;
    let serial_number = {
        let mut serial = BigNum::new()?;
        serial.rand(159, MsbOption::MAYBE_ZERO, false)?;
        serial.to_asn1_integer()?
    };
    cert_builder.set_serial_number(&serial_number)?;
    cert_builder.set_subject_name(req.subject_name())?;
    cert_builder.set_issuer_name(ca_cert.subject_name())?;
    cert_builder.set_pubkey(&key_pair)?;
    let not_before = Asn1Time::days_from_now(0)?;
    cert_builder.set_not_before(&not_before)?;
    let not_after = Asn1Time::days_from_now(365)?;
    cert_builder.set_not_after(&not_after)?;

    cert_builder.append_extension(extension::BasicConstraints::new().build()?)?;

    cert_builder.append_extension(
        extension::KeyUsage::new()
            .critical()
            .non_repudiation()
            .digital_signature()
            .key_encipherment()
            .build()?,
    )?;

    let subject_key_identifier = extension::SubjectKeyIdentifier::new()
        .build(&cert_builder.x509v3_context(Some(ca_cert), None))?;
    cert_builder.append_extension(subject_key_identifier)?;

    let auth_key_identifier = extension::AuthorityKeyIdentifier::new()
        .keyid(false)
        .issuer(false)
        .build(&cert_builder.x509v3_context(Some(ca_cert), None))?;
    cert_builder.append_extension(auth_key_identifier)?;

    let mut subject_alt_name = extension::SubjectAlternativeName::new();
    for name in dns_names {
        subject_alt_name.dns(&name);
    }
    let san = subject_alt_name.build(&cert_builder.x509v3_context(Some(ca_cert), None))?;
    cert_builder.append_extension(san)?;

    cert_builder.sign(ca_key_pair, MessageDigest::sha256())?;
    let cert = cert_builder.build();

    Ok((cert, key_pair))
}

pub fn cert_to_pem(cert: &X509, path: &str) -> Result<()> {
    let pem = cert.to_pem()?;
    let file = File::create(path)?;
    file.write_all_at(&pem, 0)?;
    Ok(())
}

pub fn key_to_pem(key: &PKey<Private>, path: &str) -> Result<()> {
    let pem = key.private_key_to_pem_pkcs8()?;
    let file = File::create(path)?;
    file.write_all_at(&pem, 0)?;
    Ok(())
}

pub fn to_pkcs12(cert: &x509::X509Ref, key: &PKeyRef<Private>) -> Result<Vec<u8>> {
    let builder = pkcs12::Pkcs12::builder();
    let pkcs12 = builder.build("", "", key, cert)?;
    match pkcs12.to_der() {
        Ok(bytes) => Ok(bytes),
        Err(e) => bail!("{:?}", e.errors()),
    }
}

pub fn cert_from_pem(path: &str) -> Result<X509> {
    let pem = read(path)?;
    match X509::from_pem(&pem) {
        Ok(cert) => Ok(cert),
        Err(e) => bail!("{:?}", e),
    }
}

pub fn key_from_pem(path: &str) -> Result<PKey<Private>> {
    let pem = read(path)?;
    match PKey::private_key_from_pem(&pem) {
        Ok(key) => Ok(key),
        Err(e) => bail!("{:?}", e),
    }
}

fn subject_name(c: &str, st: &str, l: &str, o: &str, cn: &str) -> Result<x509::X509Name> {
    let mut subj_name_builder = x509::X509NameBuilder::new()?;
    subj_name_builder.append_entry_by_text("C", c)?;
    subj_name_builder.append_entry_by_text("ST", st)?;
    subj_name_builder.append_entry_by_text("L", l)?;
    subj_name_builder.append_entry_by_text("O", o)?;
    subj_name_builder.append_entry_by_text("CN", cn)?;
    Ok(subj_name_builder.build())
}

fn days(days: u32) -> Result<(Asn1Time, Asn1Time)> {
    Ok((Asn1Time::days_from_now(0)?, Asn1Time::days_from_now(days)?))
}
