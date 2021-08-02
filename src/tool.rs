use crate::spider::ascii_to_char;
use serde_json::Result;
use serde_json::Value;
enum Tcp {
    Ss,
    V2,
}

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
fn ascii_to_string(code: Vec<u8>) -> String {
    let mut output: String = String::new();
    for cor in code.into_iter() {
        output.push(ascii_to_char(cor));
    }
    output
}
fn type_of_url(url: String) -> Tcp {
    for pair in url.chars() {
        if pair == 's' {
            return Tcp::Ss;
        }
        if pair == 'v' {
            return Tcp::V2;
        }
    }
    Tcp::Ss
}
impl Urls {
    pub fn new(url: String) -> Urls {
        let func = type_of_url(url.clone());
        match func {
            Tcp::Ss => Urls {
                urls: url,
                func: "\"ss\"".to_string(),
                add: "\"unknown\"".to_string(),
                aid: "\"unknown\"".to_string(),
                host: "\"unknown\"".to_string(),
                id: "\"unknown\"".to_string(),
                net: "\"unknown\"".to_string(),
                path: "\"unknown\"".to_string(),
                port: "\"unknown\"".to_string(),
                ps: "\"unknown\"".to_string(),
                tls: "\"unknown\"".to_string(),
                typpe: "\"unknown\"".to_string(),
            },
            Tcp::V2 => {
                let newurl = &url[8..];
                let json = ascii_to_string(base64::decode(newurl.to_string().as_bytes()).unwrap());
                let v: Result<Value> = serde_json::from_str(json.as_str());
                match v {
                    Ok(input) => {
                        Urls {
                            //company : input["add"].to_string(),
                            urls: url,
                            func: "\"vmess\"".to_string(),
                            add: input["add"].to_string(),
                            aid: input["aid"].to_string(),
                            host: input["host"].to_string(),
                            id: input["id"].to_string(),
                            net: input["net"].to_string(),
                            path: input["path"].to_string(),
                            port: input["port"].to_string(),
                            ps: input["ps"].to_string(),
                            tls: input["tls"].to_string(),
                            typpe: input["type"].to_string(),
                        }
                    }
                    Err(_) => Urls {
                        urls: url,
                        func: "\"vmess\"".to_string(),
                        add: "\"unknown\"".to_string(),
                        aid: "\"unknown\"".to_string(),
                        host: "\"unknown\"".to_string(),
                        id: "\"unknown\"".to_string(),
                        net: "\"unknown\"".to_string(),
                        path: "\"unknown\"".to_string(),
                        port: "\"unknown\"".to_string(),
                        ps: "\"unknown\"".to_string(),
                        tls: "\"unknown\"".to_string(),
                        typpe: "\"unknown\"".to_string(),
                    },
                }
            }
        }
    }
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
    ///获取运行时候的json
    pub fn get_the_json(&self) -> String {
        let temp = self.port.clone();
        let length = temp.len();
        let port: String = (&temp[1..length - 1]).to_string();
        let temp2 = self.aid.clone();
        let length2 = temp2.len();
        let aid: String = (&temp2[1..length2 - 1]).to_string();
        format!(
            "{{
    \"inbounds\":[{{
        \"port\":8889,
        \"listen\":\"127.0.0.1\",
        \"protocol\":\"http\",
        \"settings\":{{
            \"udp\": true
        }}
    }}],
    \"outbounds\":[{{
        \"protocol\":{},
        \"sendThrough\": \"0.0.0.0\",
        \"settings\":{{
            \"vnext\": [{{
                \"address\": {},
                \"port\":{},
                \"users\":[{{
                    \"alterId\": {},
                    \"id\":{}
                }}]
            }}]
        }},
        \"streamSettings\":{{
            \"dsSettings\": {{
                \"path\": {}
            }},
            \"httpSettings\":{{
                \"host\": [
                ],
                \"path\":{}
            }},
            \"kcpSettings\": {{
                \"congestion\": false,
                \"downlinkCapacity\":20,
                \"header\": {{
                    \"type\": \"none\"
                }},
                \"mtu\": 1350,
                \"readBufferSize\": 1,
                \"tti\": 20,
                \"uplinkCapacity\": 5,
                \"writeBufferSize\": 1
            }},
            \"network\": {},
            \"quicSettings\":{{
                \"header\": {{
                    \"type\":\"none\"
                }},
                \"key\": \"\",
                \"security\":\"\"
            }},
            \"security\":\"none\",
            \"sockopt\":{{
                \"mark\": 255,
                \"tcpFastOpen\": false,
                \"tproxy\": \"off\"
            }},
            \"tcpSettings\": {{
                \"header\": {{
                    \"request\" :{{
                        \"headers\":{{
                        }},
                        \"method\": \"GET\",
                        \"path\":[
                        ],
                        \"version\":\"1.1\"
                    }},
                    \"type\": \"none\"
                }}
            }},
            \"tlsSettings\": {{
                \"allowInsecure\": true,
                \"allowInsecureCiphers\": true,
                \"alpn\":[
                ],
                \"certificates\":[
                ],
                \"disableSessionResumption\":true,
                \"disableSystemRoot\":true,
                \"serveName\": \"\"
            }},
            \"wsSettings\" :{{
                \"headers\" :{{
                }},
                \"path\":{}
            }},
            \"xtlsSettings\":{{
                \"allowInsecure\":true,
                \"allowInsecureCiphers\":true,
                \"alpn\":[
                ],
                \"certificates\":[
                ],
                \"disableSessionResumption\": false,
                \"disableSystemRoot\": true,
                \"serveName\":\"\"
            }},
            \"tag\":\"outBound_PROXY\"
        }}
    }},
    {{
        \"protocol\":\"freedom\",
        \"tag\": \"direct\",
        \"settings\":{{}}
    }}],
    \"routing\": {{
        \"domainStrategy\": \"IPOnDemand\",
        \"rules\":[{{
            \"type\":\"field\",
            \"ip\":[\"geoip:private\"],
            \"outboundTag\": \"direct\"
        }}]
    }}
}}",
            self.func, self.add, port, aid, self.id, self.path, self.path, self.net, self.path
        )
    }
}
pub fn remove_quotation(input: String) -> String {
    let length = input.len();
    (&input[1..length - 1]).to_string()
}
