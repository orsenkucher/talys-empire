use plotters::prelude::*;
use regex::Regex;
use std::error::Error;
use std::fmt::{Debug, Display};
use std::{fmt, fs, io, mem};
use std::{iter::FromIterator, str::FromStr};

const TRANSITIONS: &str = "114Cd_cupture_gamma_spectra.dat";
const EMPIRE_TALYS: &str = "Cd113_ng_spectra_EMP_TAL_1eV.txt";

pub struct Core {
    directory: String,
}

#[derive(Debug, Clone, PartialEq, Default)]
struct Value {
    value: f64,
    delta: f64,
}

#[derive(Debug, PartialEq, Default)]
struct Transition {
    energy: Value,    // keV
    intensity: Value, // 1/10000n
}

#[derive(Debug, PartialEq, Default)]
struct Theoretical {
    energy: f64, // MeV
    empire: f64, // mb/MeV
    talys: f64,  // mb/MeV
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

    fn read_empire_talys(&self) -> io::Result<String> {
        fs::read_to_string(format!("{}/{}", self.directory, EMPIRE_TALYS))
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

    fn theoretical(lines: &Vec<&str>) -> Vec<Theoretical> {
        lines
            .iter()
            .map(|line| line.split_whitespace().collect())
            .collect()
    }

    fn thermal(theor: &Vec<Theoretical>) -> Vec<Theoretical> {
        theor.iter().map(Theoretical::as_termal).collect()
    }

    pub fn convert(&self) -> Result<(), Box<dyn Error>> {
        let transitions = self.read_transitions()?;
        let trimmed = Self::trim_contents(&transitions);
        let formatted = Self::format_lines(&trimmed);
        self.write_preprocessed(TRANSITIONS, &formatted)?;
        let transitions = Self::transitions(&formatted);
        self.write_processed(TRANSITIONS, &transitions)?;
        let empire_talys = self.read_empire_talys()?;
        let trimmed = Self::trim_contents(&empire_talys);
        let theoretical = Self::theoretical(&trimmed);
        let thermal = Self::thermal(&theoretical);
        let low_thermal = Theoretical::low_energy(thermal);
        let consts = Self::norm_constants(&transitions, &low_thermal);
        let (exp_talys, exp_empire) = Self::norm_exp(&transitions, consts);
        self.write_processed("exp_talys_normed.dat", &exp_talys)?;
        self.write_processed("exp_empire_normed.dat", &exp_empire)?;
        Self::plot(&exp_talys, &exp_empire)?;
        Self::plot2(&low_thermal)
    }

    fn norm_exp(
        transitions: &Vec<Transition>,
        (tal, emp): (f64, f64),
    ) -> (Vec<Transition>, Vec<Transition>) {
        let tal_exp = transitions.iter().map(|tr| tr.norm(tal)).collect();
        let emp_exp = transitions.iter().map(|tr| tr.norm(emp)).collect();
        (tal_exp, emp_exp)
    }

    fn norm_constants(transitions: &Vec<Transition>, low_thermal: &Vec<Theoretical>) -> (f64, f64) {
        let pairs: Vec<_> = transitions.iter().zip(low_thermal.iter()).collect();
        let bot: f64 = pairs
            .iter()
            .map(|(tr, _)| tr.intensity.value)
            .map(|int| int.powi(2))
            .sum();

        let (up_tal, up_emp) = pairs
            .iter()
            .map(|(tr, lo)| (tr.intensity.value, lo.talys, lo.empire))
            .map(|(int, tal, emp)| (int * tal, int * emp))
            .fold((0.0, 0.0), |acc, next| (acc.0 + next.0, acc.1 + next.1));

        (up_tal / bot, up_emp / bot)
    }

    fn range<S: Fn(&Transition) -> B, B: PartialOrd>(
        transitions: &Vec<&Transition>,
        selector: S,
    ) -> (B, B) {
        let cmp = |a: &B, b: &B| a.partial_cmp(b).unwrap();
        let select = || transitions.iter().map(ToOwned::to_owned).map(&selector);
        (select().min_by(cmp).unwrap(), select().max_by(cmp).unwrap())
    }

    fn range2<S: Fn(&(f64, f64)) -> B, B: PartialOrd>(
        transitions: &Vec<&(f64, f64)>,
        selector: S,
    ) -> (B, B) {
        let cmp = |a: &B, b: &B| a.partial_cmp(b).unwrap();
        let select = || transitions.iter().map(ToOwned::to_owned).map(&selector);
        (select().min_by(cmp).unwrap(), select().max_by(cmp).unwrap())
    }

    fn plot2(theor: &Vec<Theoretical>) -> Result<(), Box<dyn Error>> {
        let root =
            BitMapBackend::new("plt/theoretical.png", (3840 / 2, 2160 / 2)).into_drawing_area();
        root.fill(&WHITE)?;

        let tal_vals: Vec<_> = theor
            .iter()
            .map(|x| (x.energy * 1e3, x.talys * 1e-1))
            .collect();

        let emp_vals: Vec<_> = theor
            .iter()
            .map(|x| (x.energy * 1e3, x.empire * 1e-1))
            .collect();

        let chained: Vec<_> = tal_vals.iter().chain(emp_vals.iter()).collect();
        let x_rng = Self::range2(&chained, |tr| tr.0);
        let y_rng = Self::range2(&chained, |tr| tr.1);
        // let y_rng = Self::range2(
        //     &chained
        //         .into_iter()
        //         .filter(|tr| tr.intensity.value < 180E3)
        //         .collect(),
        //     |tr| tr.intensity.value,
        // );

        let rng = |(a, b): (f64, f64)| (a..b);
        let (x_rng, y_rng) = (rng(x_rng), rng(y_rng));
        println!("x: {:?}; y: {:?}", x_rng, y_rng);
        println!("tal len: {}", tal_vals.len());
        println!("emp len: {}", emp_vals.len());
        println!("EMP: {:#?}", emp_vals);

        let y_rng = 0.0..y_rng.end;

        let mut chart = ChartBuilder::on(&root)
            .margin(5)
            .caption("Theoretical Talys/Empire", ("sans-serif", 30).into_font())
            .set_label_area_size(LabelAreaPosition::Left, 160)
            .set_label_area_size(LabelAreaPosition::Bottom, 160)
            .set_label_area_size(LabelAreaPosition::Right, 160)
            .build_cartesian_2d(x_rng, y_rng)?;

        chart
            .configure_mesh()
            // .disable_x_mesh()
            // .disable_y_mesh()
            .axis_desc_style(("sans-serif", 24))
            .label_style(("sans-serif", 24))
            .x_label_formatter(&|x| format!("{:.0}", *x))
            .x_desc("Energy, keV")
            .y_label_formatter(&|y| format!("{:.0}", *y))
            .y_desc("Intensity, lo^4 normalized")
            .draw()?;

        chart
            .draw_series(LineSeries::new(
                tal_vals.iter().map(|tr| (tr.0, tr.1)),
                &RED,
            ))?
            .label("TALYS")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

        chart
            .draw_series(LineSeries::new(
                emp_vals.iter().map(|tr| (tr.0, tr.1)),
                // &BLUE,
                ShapeStyle::from(&BLUE).stroke_width(1),
            ))?
            .label("EMPIRE")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));

        chart
            .configure_series_labels()
            .label_font(("sans-serif", 24))
            .background_style(&WHITE.mix(0.8))
            .border_style(&BLACK)
            .draw()?;

        Ok(())
    }

    fn plot(
        exp_talys: &Vec<Transition>,
        exp_empire: &Vec<Transition>,
    ) -> Result<(), Box<dyn Error>> {
        let root =
            BitMapBackend::new("plt/exp_transitions.png", (3840 / 2, 2160 / 2)).into_drawing_area();
        root.fill(&WHITE)?;

        let chained: Vec<_> = exp_talys.iter().chain(exp_empire.iter()).collect();
        let x_rng = Self::range(&chained, |tr| tr.energy.value);
        let y_rng = Self::range(
            &chained
                .into_iter()
                .filter(|tr| tr.intensity.value < 180E3)
                .collect(),
            |tr| tr.intensity.value,
        );

        let rng = |(a, b): (f64, f64)| (a..b);
        let (x_rng, y_rng) = (rng(x_rng), rng(y_rng));
        let y_rng = 0.0..y_rng.end;

        let mut chart = ChartBuilder::on(&root)
            .margin(5)
            .caption(
                "Experimental normed Talys/Empire",
                ("sans-serif", 30).into_font(),
            )
            .set_label_area_size(LabelAreaPosition::Left, 160)
            .set_label_area_size(LabelAreaPosition::Bottom, 160)
            .set_label_area_size(LabelAreaPosition::Right, 160)
            .build_cartesian_2d(x_rng, y_rng)?;

        chart
            .configure_mesh()
            // .disable_x_mesh()
            // .disable_y_mesh()
            .axis_desc_style(("sans-serif", 24))
            .label_style(("sans-serif", 24))
            .x_label_formatter(&|x| format!("{:.0}", *x))
            .x_desc("Energy, keV")
            .y_label_formatter(&|y| format!("{:.0}", *y))
            .y_desc("Intensity, lo^4 normalized")
            .draw()?;

        chart
            .draw_series(LineSeries::new(
                exp_talys
                    .iter()
                    .map(|tr| (tr.energy.value, tr.intensity.value)),
                &RED,
            ))?
            .label("TALYS")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

        chart
            .draw_series(LineSeries::new(
                exp_empire
                    .iter()
                    .map(|tr| (tr.energy.value, tr.intensity.value)),
                // &BLUE,
                ShapeStyle::from(&BLUE).stroke_width(1),
            ))?
            .label("EMPIRE")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));

        chart
            .configure_series_labels()
            .label_font(("sans-serif", 24))
            .background_style(&WHITE.mix(0.8))
            .border_style(&BLACK)
            .draw()?;

        Ok(())
    }
}

impl Theoretical {
    const C_THERMAL: f64 = 6.2869461;
    const E_MAX: f64 = 325_000.0;
    const E_MULT: f64 = 1E6;
    fn as_termal(&self) -> Self {
        Self {
            energy: self.energy,
            empire: Self::C_THERMAL * self.empire,
            talys: Self::C_THERMAL * self.talys,
        }
    }

    fn low_energy(thermal: Vec<Self>) -> Vec<Self> {
        thermal
            .into_iter()
            .filter(|th| th.energy * Self::E_MULT < Self::E_MAX)
            .collect()
    }
}

impl Transition {
    fn norm(&self, c: f64) -> Self {
        Self {
            energy: self.energy.clone(),
            intensity: self.intensity.norm(c),
        }
    }
}

impl Value {
    fn norm(&self, c: f64) -> Self {
        Self {
            value: self.value * c,
            delta: self.delta * c,
        }
    }
}

struct Parser<I, V>(I, V);
impl<'a, I, V> Parser<I, V>
where
    I: IntoIterator<Item = &'a str>,
    V: FromStr,
    V::Err: Debug,
{
    fn value(it: &'a str) -> V {
        it.parse().expect(&format!("failed on {}", it))
    }

    fn values(iter: I) -> Vec<V> {
        iter.into_iter().map(Self::value).collect()
    }
}

impl<'a> FromIterator<&'a str> for Transition {
    fn from_iter<T: IntoIterator<Item = &'a str>>(iter: T) -> Self {
        let (energy, intensity) = match &mut Parser::values(iter)[..] {
            [e, i] => (mem::take(e), mem::take(i)),
            _ => unreachable!("error parsing transition entries"),
        };
        Self { energy, intensity }
    }
}

impl<'a> FromIterator<&'a str> for Theoretical {
    fn from_iter<T: IntoIterator<Item = &'a str>>(iter: T) -> Self {
        let (energy, empire, talys) = match &mut Parser::values(iter)[..] {
            [en, em, ta] => (mem::take(en), mem::take(em), mem::take(ta)),
            _ => unreachable!("error parsing transition entries"),
        };
        Self {
            energy,
            empire,
            talys,
        }
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

impl Display for Transition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.energy, self.intensity)?;
        Ok(())
    }
}

impl Display for Value {
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

    #[test]
    fn copy_from() {
        let mut src = vec![1, 2, 3, 4];
        let out = &mut src[..];
        let to_copy = 3;
        out[..to_copy].copy_from_slice(&vec![4, 5, 6, 7][..to_copy]);
        assert_eq!(out, vec![4, 5, 6, 4]);
    }
}
