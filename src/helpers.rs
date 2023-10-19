// `tracing` is a framework for instrumenting Rust programs to collect scoped, structured, and async-aware diagnostics
pub fn setup_subscriber() {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
}

// This function is needed because .trim() didn't work to remove the carriage return on the serial comms
pub fn parse_str_to_num(s: &str) -> String {
    let mut ret = String::new();
    let s = s.trim();
    for k in s.chars() {
        if k.is_digit(10) {
            ret.push(k);
        } else {
            break;
        }
    }

    ret.to_string()
}

pub fn _parse_console(s: String, variable: &str) -> Option<f64> {
    // Check for the key phrase before parsing
    let a = s.split(variable).collect::<Vec<&str>>();
    let b: Vec<&str> = a[1].split(" ").collect();
    let val = b[0];
    let k = parse_str_to_num(val);

    match k.parse::<f64>() {
        Ok(n) => {
            return Some(n);
        }
        Err(_) => None,
    }
}

// Only look for variables with carriage return '\n' at the end of line
// This prevents looking for partial lines.
// ie: user types 'adc1', we don't want to look for 'a' or 'ad' or 'adc' as they type
pub fn parse_vars(s: &String) -> Vec<String> {
    let mut buf: Vec<String> = Vec::new();
    let mut t = Vec::new();
    for i in s.chars() {
        if i == '\n' {
            if t.len() > 0 {
                buf.push(t.to_owned().into_iter().collect());
                t.clear();
            }
        } else {
            t.push(i);
        }
    }
    buf
}
