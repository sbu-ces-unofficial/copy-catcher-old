use std::collections::HashMap;
use std::fs::metadata;
use std::path::Path;

use crossbeam::thread;
use flume;
use walkdir::WalkDir;

#[derive(PartialEq, Eq, Hash)]
pub enum VerifyErrType {
    SrcMissing,
    DstMissing,
    SrcSmaller,
    DstSmaller,
    OK,
}

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

pub fn verify_path_with_logger(src_path: &str, dst_path: &str, log: fn(&VerifyErr)) -> VerifyErrStats {
    let mut stats = VerifyErrStats::new();
    let (tx, rx) = flume::unbounded();

    // TODO: fix unwrap
    thread::scope(|scope| {
        scope.spawn(|_| {
            // TODO: error handling
            let _ = verify_path_async(src_path, dst_path, tx);
        });
    }).unwrap();
    
    loop {
        match rx.try_recv() {
            Ok(verify_error) => {
                log(&verify_error);
                *stats.entry(verify_error.kind).or_insert(0) += 1;
            },
            Err(flume::TryRecvError::Disconnected) => break,
            Err(flume::TryRecvError::Empty) => {},
        }
    }

    stats
}

pub fn verify_path(src_path: &str, dst_path: &str) -> Vec<VerifyErr> {
    let mut result = Vec::new();
    let (tx, rx) = flume::unbounded();

    // TODO: fix unwrap
    thread::scope(|scope| {
        scope.spawn(|_| {
            // TODO: error handling
            let _ = verify_path_async(src_path, dst_path, tx);
        });
    }).unwrap();

    loop {
        match rx.try_recv() {
            Ok(verify_error) => result.push(verify_error),
            Err(flume::TryRecvError::Disconnected) => break,
            Err(flume::TryRecvError::Empty) => {},
        }
    }

    result
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
