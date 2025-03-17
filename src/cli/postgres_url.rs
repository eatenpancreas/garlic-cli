use percent_encoding::{percent_decode_str, percent_encode, NON_ALPHANUMERIC};
use std::{net::TcpStream, str::Chars, time::Duration};

#[derive(Clone, Debug, PartialEq)]
pub struct PostgresUrl {
    pub username: String,
    pub password: Option<String>,
    pub host: String,
    pub port: u16,
    pub database: String,
}

impl PostgresUrl {
    /// input: "postgres://USERNAME:PASSWORD@localhost:5432/DB"
    pub fn parse_env_connection_string(url: &str) -> PostgresUrl {
        let mut chars = url.chars();
        assert_eq!(&parse_until(&mut chars, "://"), "postgres");

        let username_pass = parse_until(&mut chars, "@");
        let (username, password) = username_pass.split_once(":").unzip();

        PostgresUrl {
            username: decode(username.unwrap_or(&username_pass)),
            password: password.map(|pwd| decode(pwd)),
            host: parse_until(&mut chars, ":"),
            port: parse_until(&mut chars, "/")
                .parse()
                .expect("expected uInt port"),
            database: parse_until(&mut chars, ""),
        }
    }

    pub fn to_env_connection_string(&self) -> String {
        let uname = match &self.password {
            Some(pwd) if !pwd.is_empty() => format!("{}:{}", encode(&self.username), encode(&pwd)),
            _ => encode(&self.username),
        };

        format!(
            "postgres://{uname}@{}:{}/{}",
            self.host, self.port, self.database
        )
    }

    pub fn to_connection_params(&self) -> String {
        let uname = match &self.password {
            Some(pwd) if !pwd.is_empty() => {
                format!("{} password={}", encode(&self.username), encode(&pwd))
            }
            _ => encode(&self.username),
        };

        format!("host={} port={} user={uname}", self.host, self.port)
    }

    pub fn is_running(&self) -> bool {
        match TcpStream::connect((&*self.host, self.port)) {
            Ok(stream) => {
                // Set read timeout to avoid hanging
                let _ = stream.set_read_timeout(Some(Duration::from_millis(500)));

                // PostgreSQL starts with a specific protocol version message
                // But just checking if the port is open is often enough
                true
            }
            Err(_) => false,
        }
    }
}

fn parse_until(chars: &mut Chars, until: &str) -> String {
    let mut output = String::new();
    for ch in chars {
        output.push(ch);
        if !until.is_empty() && output.ends_with(until) {
            let _ = output.split_off(output.len() - until.len());
            break;
        }
    }

    output
}

fn encode(string: &str) -> String {
    percent_encode(string.as_bytes(), NON_ALPHANUMERIC).to_string()
}

fn decode(string: &str) -> String {
    percent_decode_str(string)
        .decode_utf8()
        .expect("expected to be able to decode the postgres url")
        .to_string()
}
