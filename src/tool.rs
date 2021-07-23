#[allow(dead_code)]
#[derive(Clone)]
pub struct Urls {
    pub func: String,
    pub urls: String,
    pub add: String,
    pub aid: String,
    pub host: String,
    pub id: String,
    pub net: String,
    pub path: String,
    pub port: String,
    pub ps: String,
    pub tls: String,
    pub typpe: String,
}
impl Urls {
    pub fn get_the_link(&self) -> String {
        let mut temp = String::new();
        if self.func == *"\"vmess\"" {
            temp.push_str(&format!(
                "vmess://{}:{}-{}@{}:{}/#{}",
                &remove_quotation(self.net.clone()),
                &remove_quotation(self.id.clone()),
                &remove_quotation(self.aid.clone()),
                &remove_quotation(self.add.clone()),
                &remove_quotation(self.port.clone()),
                &remove_quotation(self.ps.clone())
            ))
        } else {
            temp.push_str("unknown");
        }
        temp
    }
}
pub fn remove_quotation(input: String) -> String {
    let length = input.len();
    (&input[1..length - 1]).to_string()
}
