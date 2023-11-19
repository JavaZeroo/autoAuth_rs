use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use select::document::Document;
use select::predicate::Name;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};
use serde_yaml;

// 定义配置结构
#[derive(Debug, Serialize, Deserialize)]
struct Config {
    username: String,
    password: String,
    domain: String,
}

// 定义登录器
struct Loginer {
    config: Config,
    cache: bool,
    debug: bool,
    cache_path: Option<PathBuf>,
}

impl Loginer {
    // 初始化函数
    fn new(config: Config, cache: bool, debug: bool) -> Self {
        let cache_path = if cache {
            let path = PathBuf::from(".cache");
            fs::create_dir_all(&path).unwrap();
            let cache_config = serde_yaml::to_string(&config).unwrap();
            fs::write(path.join("cache_config.yaml"), cache_config).unwrap();
            Some(path)
        } else {
            None
        };

        Loginer {
            config,
            cache,
            debug,
            cache_path,
        }
    }

    // 登录函数
    fn login(&self) -> Result<String, reqwest::Error> {
        let client = Client::new();

        // 构建 headers
        let mut headers = HeaderMap::new();
        headers.insert("Host", HeaderValue::from_static("172.17.3.10"));
        headers.insert("Connection", HeaderValue::from_static("keep-alive"));
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/x-www-form-urlencoded"));
        headers.insert("Origin", HeaderValue::from_static("http://172.17.3.10"));
        headers.insert("DNT", HeaderValue::from_static("1"));
        headers.insert("Upgrade-Insecure-Requests", HeaderValue::from_static("1"));
        headers.insert("User-Agent", HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/116.0.0.0 Safari/537.36 Edg/116.0.1938.54"));
        headers.insert("Accept", HeaderValue::from_static("text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7"));
        headers.insert("Referer", HeaderValue::from_static("http://172.17.3.10/srun_portal_pc.php?ac_id=1&"));
        headers.insert("Accept-Encoding", HeaderValue::from_static("gzip, deflate"));
        headers.insert("Accept-Language", HeaderValue::from_static("en-US,en;q=0.9"));

        
        let payload = [
            ("action", "login"),
            ("ac_id", "1"),
            ("user_ip", ""),
            ("nas_ip",""),
            ("user_mac", ""),
            ("url", ""),
            ("drop", "0"),
            ("domain", &self.config.domain),
            ("username", &self.config.username),
            ("password", &self.config.password),
            ("save_me", "1"),
        ];
        if self.debug {
            println!("{:?}", payload);
        }

        let response = client.post("http://172.17.3.10/srun_portal_pc.php")
            .headers(headers)
            .form(&payload)
            .send()?;

        let content = response.text()?;
        if self.debug {
            println!("{}", content);
        }

        if self.cache {
            if let Some(ref path) = self.cache_path {
                fs::write(path.join("response.html"), &content).unwrap();
            }
        }
        Ok(content)
    }

    // 解析函数（简单示例）
    fn parse(&self, content: &str) {
        let document = Document::from(content);

        // // 例子：查找具有特定文本开始的段落
        // for p in document.find(Name("p")).filter(|p| p.text().starts_with('E')) {
        //     println!("{}", p.text());
        //     // 这里可以添加错误处理或其他逻辑
        // }
        let mut skip = true;
        // 以解析表格为例
        if let Some(table) = document.find(Name("table")).next() {
            for row in table.find(Name("tr")) {
                let mut row_data = Vec::new();
                for cell in row.find(Name("td")) {
                    let text = cell.text().trim().to_string();
                    if !text.is_empty() {
                        row_data.push(text);
                    }
                }
                if !row_data.is_empty() {
                    if skip {
                        skip = false;
                        continue;
                    }
                    println!("{:?}", row_data);
                }
            }
        }
    }
}

fn main() {
    let config: Config = serde_yaml::from_reader(fs::File::open("config.yaml").unwrap()).unwrap();
    let loginer = Loginer::new(config, true, false);
    match loginer.login() {
        Ok(response) => {
            println!("登录成功!");
            loginer.parse(&response);
        },
        Err(e) => println!("登录失败: {}", e),
    }
}
