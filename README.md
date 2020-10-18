# .dat file fix commits
- [remove accidental ones](https://github.com/orsenkucher/talys-empire/commit/030870549beeb6cd33c3f7ec93b2f61f20bd1837) -
Видалив із файлу випадкові одинички і перевірив відповідність значень в тих місцях.
- [correct CV errors in 114Cd file](https://github.com/orsenkucher/talys-empire/commit/7930033a208795cbd70f711ab44cac9c87cb724d) -
Виправив усі помилки компьютерного перенесення таблиці в електронний вигляд, такі як буква `S` замість числа `5` або `8`, `i -> 4`, `l -> 1`;
також відредагував деякі відступи. 


# Project structure
Проект складається з трьох основних директорій:
- `/dat` - папка у якій знаходяться файли з експериментальними та теоретичними даними, а також місце куди потрапляють відформатовані дані під час роботи програми.
> Примітка: файл `.gitignore` відфільтровує такі автоматично створені файли, бо вони змінюються в ході розробки та будуть псувати наочність змін між комітами. Фінальні результати будуть знаходитись в папці `results`.
- `/plt` - місце куди потрапляють збудовані графіки.
- `/src` - тут я пишу виконувані коди. Далі розглянемо це місце детальніше:
## `/src`
Файл [`main.rs`](https://github.com/orsenkucher/talys-empire/blob/master/src/main.rs) - утримує всього одну функцію `fn main()` та є модулем, що можна запускати.  
До його задач входить підключення бібліотеки [`lib.rs`](https://github.com/orsenkucher/talys-empire/blob/master/src/lib.rs)
```rust
use talys_empire::Core;
```
та вказівка щодо знаходження директорії з `.dat` файлами
```rust 
let core = Core::new("dat");
```
Після цього викликається функція `core.convert()`, в якій зосереджена основна логіка переформатування даних.
```rust
use std::error::Error;
use talys_empire::Core;

fn main() -> Result<(), Box<dyn Error>> {
    let core = Core::new("dat");
    core.convert()?;
    Ok(())
}
```
## `lib.rs`
[`fn convert()`](https://github.com/orsenkucher/talys-empire/blob/master/src/lib.rs#L118) складається з багатьох атомарних операцій:  
1. Насамперед нам потрібно отримати вміст файлу `114Cd_cupture_gamma_spectra.dat`
```rust
let transitions = self.read_transitions()?;
```
2. Змінна `transitions` тепер тримає строку, яку необхідно підготувати до парсінгу очистивши її від пустих рядків та `#коментарів`
```rust
let trimmed = Self::trim_contents(&transitions);
```
Усередині, `trim_contents()` розбиває строку на рядки за символом `\n`, обрізає зайві відступи та фільтрує за зазначеними вище умовами
```rust
// (contents: &str) -> Vec<&str>
contents
    .lines()
    .map(|line| line.trim())
    .filter(|line| !line.is_empty())
    .filter(|line| !line.starts_with('#'))
    .collect()
```

3. Наступним етапом я розбиваю строки на слова, форматую їх
та записую проміжний результат у файл `preprocessed_114Cd.dat`
```rust
let formatted = Self::format_lines(&trimmed);
self.write_preprocessed(TRANSITIONS, &formatted)?;
```
Важливим кроком тут є саме форматування:
```rust
// (lines: &Vec<&str>) -> Vec<Vec<String>>
lines
    .iter()
    .map(|line| Self::glue_bracket(line))
    .map(|line| line.split_whitespace().map(Self::format_values).collect())
    .collect()
```
В першопочатковому файлі зустрічаються схожі на цей записи `2437.7(1) < 10`, і символ `<` відірваний від свого значення, він сприймається за окреме слово.
Мені такого не потрібно, тому `glue_bracket()` під'єднає його назад до відповідного запису. Далі
```rust
//...
.map(|line| line.split_whitespace().map(Self::format_values).collect())
//...
```
розбирає `line` на записи та форматує кожен прибираючи `<` та встановлюючи похибку рівну `0`. Виписуємо проміжний результат та перевіряємо його 
```rust
self.write_preprocessed(TRANSITIONS, &formatted)?;
```
> До речі, усі ці та майбутні перетворення перевіряються тестами у нижній частині `lib.rs`
> ``` rust 
> #[cfg(test)]
> mod tests {
>    #[test]
>    fn glue_bracket() {
>        assert_eq!("<0.10", Core::glue_bracket("<   0.10"));
>    }
>
>    #[test]
>    fn format_lines() {
>        let lines = vec!["150.0(5) <0.10", "150.0(5)  <  0.10"];
>        let lines = Core::format_lines(&lines);
>        let expect = vec!["150.0(5)", "0.10(0)"];
>        assert_eq!(lines, vec![expect.clone(), expect.clone()]);
>    }
> }
> ```
4. Нарешті все готово для переведення вмісту файлу до пам'яті комп'ютера. Для цього визначимо дві стуктури 
```rust
struct Value {
    value: f64,
    delta: f64,
}

struct Transition {
    energy: Value,
    intensity: Value,
}
```
Використовуючи попередньо отримані записи(`formatted`), конвертуємо кожен рядок в цій структурі даних в `Transition`
```rust
// fn transitions(lines: &Vec<Vec<String>>) -> Vec<Transition>
lines
    .iter()
    .map(|line| line.iter().collect())
    .collect()
```
```rust
// fn convert()
let transitions = Self::transitions(&formatted);
```
Все що відбувається в `fn transitions()`, це розбирання(`iter`) вектору на рядки та збирання(`collect`) назад вже маючи `Transition` замість рядка.  

Зберігаємо до `processed_114Cd.dat`
```rust
self.write_processed(TRANSITIONS, &transitions)?; 
```
Який вже має такий вигляд
```yaml
  60.911      2.0     0.12      2.0
  75.177      5.0   0.0027      4.0
  78.601      3.0     0.09     15.0
  80.605      3.0    0.046      5.0
```
5. Тепер зчитаємо `Cd113_ng_spectra_EMP_TAL_1eV.txt`
```rust
let empire_talys = self.read_empire_talys()?;
let trimmed = Self::trim_contents(&empire_talys);
let theoretical = Self::theoretical(&trimmed);
```
За аналогією з попереднім файлом очищаємо від коментарів та зберігаємо у заздалегідь підготовлену структуру `Theoretical`
```rust
struct Theoretical {
    energy: f64,
    empire: f64,
    talys: f64,
}
```
6. Маючи вектор теоретичних значень знаходимо гамма-спектри для теплових нейтронів
```rust
// fn convert()
let thermal = Self::thermal(&theoretical);

// fn thermal(theor) -> thermal
theor.iter().map(Theoretical::as_termal).collect()

// impl Theoretical
const C_THERMAL: f64 = 6.2869461;
fn as_termal(&self) -> Self {
    Self {
        energy: self.energy,
        empire: Self::C_THERMAL * self.empire,
        talys: Self::C_THERMAL * self.talys,
    }
}
```
7. Залишаємо тільки значення гамма-спектрів при низьких енергіях
```rust
// fn convert()
let low_thermal = Theoretical::low_energy(thermal);

// impl Theoretical
const E_MAX: f64 = 325_000.0;
const E_MULT: f64 = 1E6;
fn low_energy(thermal: Vec<Self>) -> Vec<Self> {
    thermal
        .into_iter()
        .filter(|th| th.energy * Self::E_MULT < Self::E_MAX)
        .collect()
}
```
8. Обчислюємо нормувальну константу методом найменших квадратів
```rust
let consts = Self::norm_constants(&transitions, &low_thermal);
```
Розберемо розрахунки у `fn norm_constants()` детальніше:
* Зіпуємо `transitions` з `low_thermal` отримуючи `pairs` - вектор кортежів `(&Transition, &Theoretical)`
```rust
// fn norm_constants(transitions: &Vec<Transition>, low_thermal: &Vec<Theoretical>) -> (f64, f64)
let pairs: Vec<_> = transitions.iter().zip(low_thermal.iter()).collect();
```
* розраховуємо знаменник(`bot`)
```rust
let bot: f64 = pairs
    .iter()
    .map(|(tr, _)| tr.intensity.value)
    .map(|int| int.powi(2))
    .sum();
```
* та два чисельника для `TALYS` і `EMPIRE` відповідно 
```rust
let (up_tal, up_emp) = pairs
    .iter()
    .map(|(tr, lo)| (tr.intensity.value, lo.talys, lo.empire))
    .map(|(int, tal, emp)| (int * tal, int * emp))
    .fold((0.0, 0.0), |acc, next| (acc.0 + next.0, acc.1 + next.1));
```
* Повертаємо кортеж з розрахованими нормувальними константами
```rust
(up_tal / bot, up_emp / bot)
```
9. Після чого віднормовуємо гамма-інтенсивності на значення спектрів з `TALYS` та `EMPIRE`
```rust
let (exp_talys, exp_empire) = Self::norm_exp(&transitions, consts);
```
Тут `consts` це кортеж, тому назад отримуємо теж кортеж.  
Так `fn norm_exp()` визиває `fn norm()` на кожній `Transition`, що в свою чергу визиває `fn norm()` у своєму `Value`. В результаті отримуємо віднормовані інтенсивності `exp_talys`, `exp_empire`.  

10. Нарешті зберігаємо файли  
`processed_exp_talys_normed.dat`  
`processed_exp_empire_normed.dat`
```rust
self.write_processed("exp_talys_normed.dat", &exp_talys)?;
self.write_processed("exp_empire_normed.dat", &exp_empire)?;
```

### Plotting
Для побудови графіків я обрав опенсорсний крейт(`crate`) [plotters](https://crates.io/crates/plotters), ось він на [github](https://github.com/38/plotters).  

Останнім викликом функції `fn convert()` є `fn plot()`, що отримує два вектори віднормованих спектрів, з яких створює файл `exp_transitions.png` в папці `plt/` - це і є побудований графік
```rust
Self::plot(&exp_talys, &exp_empire)
```
У самій `fn plot()` створюється растровий бекенд
```rust
let root = BitMapBackend::new("plt/exp_transitions.png", (3840, 2160)).into_drawing_area();
```
Тут я зливаю два вектори в один для знаходження границь зображення
```rust
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
```
Та знаходжу їх обрізаючи значення `y` на відмітці `180,000`.  
Далі налагоджую `chart`
```rust
let mut chart = ChartBuilder::on(&root)
    //...
    .build_cartesian_2d(x_rng, y_rng)?;
```
Форматування міток на осях
```rust
chart
    .configure_mesh()
    //...
    .x_label_formatter(&|x| format!("{:.0}", *x))
    .x_desc("Energy, keV")
    .y_label_formatter(&|y| format!("{:.0}", *y))
    .y_desc("Intensity, lo^4 normalized")
    .draw()?;
```
Будую спектр `exp_talys`, а потім так само для `exp_empire` 
```rust
chart
    .draw_series(LineSeries::new(
        exp_talys
            .iter()
            .map(|tr| (tr.energy.value, tr.intensity.value)),
        &RED,
    ))?
    .label("TALYS")
```
Фінальні налаштування
```rust
chart
    .configure_series_labels()
    .label_font(("sans-serif", 24))
    .background_style(&WHITE.mix(0.8))
    .border_style(&BLACK)
    .draw()?;
```
та маю збережений `exp_transitions.png`.

# Developer Diary
З `06.07.20` по `15.10.20` - [проводив підготовку](https://github.com/orsenkucher/rustbook/commits/master) до написання літньої практики, а саме вчив мову програмування на якій буду писати, прочитав книгу [The Rust Programming Language](https://doc.rust-lang.org/book/) та декілька допоміжних джерел. Увесь процесс моєї підготовки [залогований в історіі](https://github.com/orsenkucher/rustbook/commits/master) на [github](https://github.com/orsenkucher/rustbook).  

З `15.10.20` по `18.10.20` - виконував літню практику, що відображено у цьому [документі](https://github.com/orsenkucher/talys-empire/blob/master/README.md), а також в [історії](https://github.com/orsenkucher/talys-empire/commits/master) цього [репозиторія](https://github.com/orsenkucher/talys-empire).  

Кучер Арсеній, 18.10.20  
4 курс, ФВЕ
