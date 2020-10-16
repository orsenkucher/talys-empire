use regex::Regex;
use std::error::Error;
use std::{fmt, fs, io, mem};
use std::{iter::FromIterator, str::FromStr};

const TRANSITIONS: &str = "114Cd_cupture_gamma_spectra.dat";

pub struct Core {
    directory: String,
}

#[derive(Debug, PartialEq, Default)]
struct Value {
    value: f64,
    delta: f64,
}

#[derive(Debug, PartialEq, Default)]
struct Transition {
    energy: Value,
    intensity: Value,
}

impl Core {
    pub fn new(dir: &str) -> Self {
        Self {
            directory: String::from(dir),
        }
    }

    fn read_transitions(&self) -> io::Result<String> {
        fs::read_to_string(format!("{}/{}", self.directory, TRANSITIONS))
    }

    fn write_processed(&self, file: &str, trans: &Vec<Transition>) -> io::Result<()> {
        let path = format!("{}/{}_{}", self.directory, "processed", file);
        let contents: String = trans.iter().map(|tr| format!("{}\n", tr)).collect();
        fs::write(path, contents)
    }

    fn write_preprocessed(&self, file: &str, contents: &Vec<Vec<String>>) -> io::Result<()> {
        let path = format!("{}/{}_{}", self.directory, "preprocessed", file);
        fs::write(path, Self::flatten_preprocessed(contents))
    }

    fn flatten_preprocessed(contents: &Vec<Vec<String>>) -> String {
        contents
            .iter()
            .map(|v| v.join("\t"))
            .fold(String::new(), |acc, next| acc + &next + "\n")
    }

    fn trim_contents(contents: &str) -> Vec<&str> {
        contents
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .filter(|line| !line.starts_with('#'))
            .collect()
    }

    fn format_lines(lines: &Vec<&str>) -> Vec<Vec<String>> {
        lines
            .iter()
            .map(|line| Self::glue_bracket(line))
            .map(|line| line.split_whitespace().map(Self::format_values).collect())
            .collect()
    }

    fn glue_bracket(value: &str) -> String {
        let mut value = String::from(value);
        while value.contains("< ") {
            value = value.replace("< ", "<");
        }
        value
    }

    fn format_values(value: &str) -> String {
        if value.starts_with('<') {
            format!("{}(0)", value.replace("<", "").trim())
        } else {
            String::from(value)
        }
    }

    fn transitions(lines: &Vec<Vec<String>>) -> Vec<Transition> {
        lines
            .iter()
            .map(|line| line.iter().map(AsRef::as_ref).collect())
            .collect()
    }

    pub fn convert(&self) -> Result<(), Box<dyn Error>> {
        let transitions = self.read_transitions()?;
        let trimmed = Self::trim_contents(&transitions);
        let formatted = Self::format_lines(&trimmed);
        self.write_preprocessed(TRANSITIONS, &formatted)?;
        let transitions = Self::transitions(&formatted);
        self.write_processed(TRANSITIONS, &transitions)?;
        // println!("{:?}", transitions);
        Ok(())
    }
}

impl<'a> FromIterator<&'a str> for Transition {
    fn from_iter<T: IntoIterator<Item = &'a str>>(iter: T) -> Self {
        let mut vals: Vec<_> = iter
            .into_iter()
            .map(|it| it.parse().expect(&format!("failed on {}", it)))
            .collect();
        let (energy, intensity) = match &mut vals[..] {
            [e, i] => (mem::take(e), mem::take(i)),
            _ => unreachable!("error parsing transition entries"),
        };
        Self { energy, intensity }
    }
}

impl FromStr for Value {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let re = Regex::new(r"(?P<value>.*?)\((?P<delta>.*?)\)")?;
        let cap = re.captures(s).ok_or("no matches")?;
        let value = cap["value"].parse()?;
        let delta = cap["delta"].parse()?;
        Ok(Self { value, delta })
    }
}

impl fmt::Display for Transition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.energy, self.intensity)?;
        Ok(())
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:8.*} {:8.*}",
            float_precision(&self.value),
            self.value,
            float_precision(&self.delta),
            self.delta
        )?;
        Ok(())
    }
}

fn float_precision(float: &f64) -> usize {
    let s = float.to_string();
    s.find('.').and_then(|i| Some(s.len() - i - 1)).unwrap_or(1)
}

//cargo test -- --nocapture
#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;

    #[test]
    fn whitespace_split() {
        let line = "  a b   c     d   "
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");

        assert_eq!("a b c d", line);
    }

    #[test]
    fn parse_value() {
        let value: Value = "170.857(15.15)".parse().unwrap();
        assert_eq!(
            Value {
                value: 170.857,
                delta: 15.15,
            },
            value
        );
    }

    #[test]
    fn parse_zero_value() {
        let value: Value = "0.0(0.0)".parse().unwrap();
        assert_eq!(Value::default(), value);

        let value: Value = "0(0)".parse().unwrap();
        assert_eq!(Value::default(), value);
    }

    #[test]
    fn format_values() {
        assert_eq!(Core::format_values("<0.03"), "0.03(0)");
        assert_eq!(Core::format_values("< 10"), "10(0)");
        assert_eq!(Core::format_values("<"), "(0)");
    }

    #[test]
    fn format_lines() {
        let lines = vec!["150.0(5) <0.10", "150.0(5)  <  0.10"];
        let lines = Core::format_lines(&lines);
        let expect = vec!["150.0(5)", "0.10(0)"];
        assert_eq!(lines, vec![expect.clone(), expect.clone()]);
    }

    #[test]
    fn glue_bracket() {
        assert_eq!("<0.10", Core::glue_bracket("<   0.10"));
    }

    #[test]
    fn collect_transition() {
        let transition = "170.857(15)    0.045(9)"
            .split_whitespace()
            .collect::<Transition>();

        assert_eq!(
            Transition {
                energy: Value {
                    value: 170.857,
                    delta: 15.0
                },
                intensity: Value {
                    value: 0.045,
                    delta: 9.0
                }
            },
            transition
        );
    }

    #[test]
    fn regex_use() {
        let re = Regex::new(r"(?P<value>.*?)\((?P<delta>.*?)\)").unwrap();
        let cap = re.captures("170.857(15)").unwrap();
        let value = cap["value"].parse().unwrap();
        let delta = cap["delta"].parse().unwrap();
        assert_eq!(170.857, value);
        assert_eq!(15f64, delta);
    }

    #[test]
    fn vec_remove() {
        let mut v = vec!["a", "b"];
        let a = v.remove(0);
        let b = v.remove(0);
        assert_eq!(a, "a");
        assert_eq!(b, "b");
    }

    #[test]
    fn precision() {
        assert_eq!(12.0.to_string(), "12");
        assert_eq!(float_precision(&12.0), 1);
        assert_eq!(float_precision(&12.0), 1);
        assert_eq!(float_precision(&12.1), 1);
        assert_eq!(float_precision(&12.01), 2);
        assert_eq!(float_precision(&12.000_001), 6);
        assert_eq!(float_precision(&0.000_001), 6);
        assert_eq!(float_precision(&0.000_000), 1);
        assert_eq!(float_precision(&1.01e2), 1);
        assert_eq!(float_precision(&1.0001e2), 2);
    }
}
