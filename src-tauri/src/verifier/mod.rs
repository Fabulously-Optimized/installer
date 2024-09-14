use std::{
    ffi::{c_char, CString, NulError},
    ptr::null_mut,
};

use thiserror::Error;

extern "C" {
    fn verifier_verify(digestHexRaw: *const c_char, bundlePathRaw: *const c_char) -> *mut c_char;
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Unexpected NUL byte in args: {0}")]
    NulError(NulError),
    #[error("Verification Failure: {0}")]
    VerifierError(String),
}

pub fn verify(digest_hex: &str, bundle_path: &str) -> Result<(), Error> {
    let digest_hex_raw = CString::new(digest_hex).map_err(Error::NulError)?;
    let bundle_path_raw = CString::new(bundle_path).map_err(Error::NulError)?;
    let result = unsafe { verifier_verify(digest_hex_raw.as_ptr(), bundle_path_raw.as_ptr()) };
    if result == null_mut() {
        Ok(())
    } else {
        unsafe {
            Err(Error::VerifierError(
                CString::from_raw(result).to_string_lossy().to_string(),
            ))
        }
    }
}
