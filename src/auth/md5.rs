use md5;

pub fn hash_password(password: &str) -> String {
    let hash = md5::compute(password);
    format!("{:x}", hash)
}