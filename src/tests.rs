use crate::PostgresUrl;

#[test]
fn test_parse_postgres_url() {
    let url =
        PostgresUrl::parse_env_connection_string("postgres://USERNAME:PASSWORD@localhost:5432/DB");
    assert_eq!(
        url,
        PostgresUrl {
            username: "USERNAME".to_owned(),
            password: Some("PASSWORD".to_owned()),
            host: "localhost".to_owned(),
            port: 5432,
            database: "DB".to_owned()
        }
    );
}

#[test]
fn test_connection_string_output() {
    let string = "postgres://USERNAME:PASSWORD@localhost:5432/DB";
    let url = PostgresUrl::parse_env_connection_string(string);
    assert_eq!(url.to_env_connection_string(), string)
}

#[test]
fn test_connection_string_output_2() {
    let string = "postgres://USERNAME@localhost:5432/DB";
    let url = PostgresUrl::parse_env_connection_string(string);
    assert_eq!(url.to_env_connection_string(), string)
}

#[test]
fn test_url_to_params() {
    let url =
        PostgresUrl::parse_env_connection_string("postgres://USERNAME:PASSWORD@localhost:5432/DB");
    assert_eq!(
        "host=localhost port=5432 user=USERNAME password=PASSWORD",
        url.to_connection_params()
    );
}

#[test]
fn test_url_to_params_2() {
    let url = PostgresUrl::parse_env_connection_string("postgres://USERNAME@localhost:5432/DB");
    assert_eq!(
        "host=localhost port=5432 user=USERNAME",
        url.to_connection_params()
    );
}

#[test]
fn test_parse_postgres_url_without_pwd() {
    let url = PostgresUrl::parse_env_connection_string("postgres://USERNAME@localhost:5432/DB");
    assert_eq!(
        url,
        PostgresUrl {
            username: "USERNAME".to_owned(),
            password: None,
            host: "localhost".to_owned(),
            port: 5432,
            database: "DB".to_owned()
        }
    );
}
