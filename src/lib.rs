use std::fs::metadata;

pub enum VerifyErrType {
    SrcMissing,
    DstMissing,
    SrcSmaller,
    DstSmaller,
    OK,
}

type VerifyErrMessage = String;

pub struct VerifyErr {
    pub kind: VerifyErrType,
    pub message: VerifyErrMessage,
}

pub fn verify(src: &str, dst: &str) -> VerifyErr {
    let src_size = match metadata(src) {
        Ok(src_metadata) => src_metadata.len(),
        Err(_) => return VerifyErr{
            kind: VerifyErrType::SrcMissing,
            message: format!("The source, {}, is missing!", src),
        }
    };

    let dst_size = match metadata(dst) {
        Ok(dst_metadata) => dst_metadata.len(),
        Err(_) => return VerifyErr{
            kind: VerifyErrType::DstMissing,
            message: format!("The destination, {}, is missing!", dst),
        }
    };

    if src_size < dst_size {
        return VerifyErr{
            kind: VerifyErrType::SrcSmaller,
            message: format!("The source, {}, is smaller than the destination, {}, by {} bytes.", src, dst, dst_size - src_size),
        };
    } else if src_size > dst_size {
        return VerifyErr{
            kind: VerifyErrType::DstSmaller,
            message: format!("The source, {}, is bigger than the destination, {}, by {} bytes.", src, dst, src_size - dst_size),
        };
    } else {
        return VerifyErr{
            kind: VerifyErrType::OK,
            message: format!("The source, {}, matches with the destination, {}", src, dst),
        };
    }
}
