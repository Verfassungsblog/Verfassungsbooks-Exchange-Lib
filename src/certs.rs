use std::fs::File;
use std::io::BufReader;
use tokio_rustls::rustls::pki_types::{CertificateDer, CertificateRevocationListDer, PrivateKeyDer};
use tokio_rustls::rustls::RootCertStore;

pub fn load_root_ca(path: String) -> RootCertStore {
    // Load certificates
    let mut root_store = tokio_rustls::rustls::RootCertStore::empty();
    let ca_file = File::open(&path).expect("cannot open CA file");
    let mut reader = BufReader::new(ca_file);
    let certs: Vec<_> = rustls_pemfile::certs(&mut reader).map(|cert| cert.expect("Couldn't parse root CA")).collect();
    for cert in certs{
        root_store.add(cert).expect("Couldn't add CA file to root store.");
    }
    root_store
}

pub fn load_client_cert(path: String) -> Vec<CertificateDer<'static>>{
    let file = File::open(&path).expect("cannot open client cert file");
    let mut reader = BufReader::new(file);
    let certs = rustls_pemfile::certs(&mut reader);
    let certs = certs.map(|cert|cert.expect("Couldn't parse cert file")).collect();
    certs
}

pub fn load_private_key(path: String) -> PrivateKeyDer<'static>{
    let file = File::open(&path).expect("cannot open client key file");
    let mut reader = BufReader::new(file);
    rustls_pemfile::private_key(&mut reader).expect("Couldn't parse Private key file!").expect("Missing private key")
}

pub fn load_crl(path: String) -> Vec<CertificateRevocationListDer<'static>>{
    let crl_file = File::open(path).expect("Failed to open CRL file");
    let mut crl_reader = BufReader::new(crl_file);
    let res = rustls_pemfile::crls(&mut crl_reader).map(|cert|cert.expect("Couldn't load CRL!")).collect();

    res
}