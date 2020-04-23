#![feature(trait_alias)]

use std::collections::HashMap;
use std::fs::metadata;
use std::path::Path;

use flume;
use std::thread;
use walkdir::WalkDir;

pub mod files;

#[derive(PartialEq, Eq, Hash)]
pub enum VerifyErrType {
    SrcMissing,
    DstMissing,
    SrcSmaller,
    DstSmaller,
    OK,
}

pub trait VerifyErrFilterer = Fn(&VerifyErrType) -> bool;
pub trait VerifyErrLogger = FnMut(&VerifyErr);
type VerifyErrMessage = String;
type VerifyErrStats = HashMap<VerifyErrType, u64>;

pub struct VerifyErr {
    pub kind: VerifyErrType,
    pub message: VerifyErrMessage,
}

pub fn verify_path_async(src_path: &str, dst_path: &str, sender: flume::Sender<VerifyErr>) -> Result<(), flume::SendError<VerifyErr>> {
    for entry in WalkDir::new(src_path)
        .contents_first(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if !entry.file_type().is_dir() {
            let src = entry.path().to_str().unwrap();
            let base_path = entry.path().strip_prefix(src_path).unwrap();
            let dst_path = Path::new(dst_path).join(base_path);
            let dst = dst_path.to_str().unwrap();

            let verify_err = verify(src, dst);
            sender.send(verify_err)?
        }
    }
    Ok(())
}

pub fn verify_path_with_logger<L>(src_path: &str, dst_path: &str, log: L) -> VerifyErrStats
where L: VerifyErrLogger {
    verify_path_with_filtered_logger(src_path, dst_path, log, verify_err_accept_all_filter)
}

pub fn verify_path_with_filtered_logger<F, L>(src_path: &str, dst_path: &str, mut log: L, filter: F) -> VerifyErrStats
where F: VerifyErrFilterer, L: VerifyErrLogger {
    let mut stats = VerifyErrStats::new();
    let (tx, rx) = flume::unbounded();

    let src_path = src_path.to_owned().clone();
    let dst_path = dst_path.to_owned().clone();
    thread::spawn(move || {
        // TODO: error handling
        let _ = verify_path_async(&src_path, &dst_path, tx);
    });
    
    loop {
        match rx.try_recv() {
            Ok(verify_error) => {
                if filter(&verify_error.kind) {
                    log(&verify_error);
                }
                *stats.entry(verify_error.kind).or_insert(0) += 1;
            },
            Err(flume::TryRecvError::Disconnected) => break,
            Err(flume::TryRecvError::Empty) => {},
        }
    }

    stats
}

pub fn verify_path_filtered<F>(src_path: &str, dst_path: &str, filter: F) -> Vec<VerifyErr>
where F: VerifyErrFilterer {
    let mut result = Vec::new();
    let (tx, rx) = flume::unbounded();

    let src_path = src_path.to_owned().clone();
    let dst_path = dst_path.to_owned().clone();
    thread::spawn(move || {
        // TODO: error handling
        let _ = verify_path_async(&src_path, &dst_path, tx);
    });

    loop {
        match rx.try_recv() {
            Ok(verify_error) => {
                if filter(&verify_error.kind) {
                    result.push(verify_error);
                }
            },
            Err(flume::TryRecvError::Disconnected) => break,
            Err(flume::TryRecvError::Empty) => {},
        }
    }

    result
}

pub fn verify_path(src_path: &str, dst_path: &str) -> Vec<VerifyErr> {
    verify_path_filtered(src_path, dst_path, verify_err_accept_all_filter)
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

pub fn verify_err_accept_all_filter(_verify_err_type: &VerifyErrType) -> bool {
    true
}

pub fn verify_err_only_errors_filter(verify_err_type: &VerifyErrType) -> bool {
    *verify_err_type != VerifyErrType::OK
}
