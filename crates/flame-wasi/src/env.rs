//! WASI environ_get / environ_sizes_get / args_get / args_sizes_get.

pub fn args_sizes(args: &[String]) -> (u32, u32) {
    let count = args.len() as u32;
    let buf_size: u32 = args.iter().map(|a| a.len() as u32 + 1).sum();
    (count, buf_size)
}

pub fn environ_sizes(env: &[(String, String)]) -> (u32, u32) {
    let count = env.len() as u32;
    let buf_size: u32 = env
        .iter()
        .map(|(k, v)| k.len() as u32 + 1 + v.len() as u32 + 1)
        .sum();
    (count, buf_size)
}

/// Serialize args into a byte buffer (null-terminated strings).
pub fn serialize_args(args: &[String]) -> Vec<u8> {
    let mut buf = Vec::new();
    for a in args {
        buf.extend_from_slice(a.as_bytes());
        buf.push(0);
    }
    buf
}

/// Serialize env vars as `KEY=VALUE\0` byte buffer.
pub fn serialize_env(env: &[(String, String)]) -> Vec<u8> {
    let mut buf = Vec::new();
    for (k, v) in env {
        buf.extend_from_slice(k.as_bytes());
        buf.push(b'=');
        buf.extend_from_slice(v.as_bytes());
        buf.push(0);
    }
    buf
}
