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
Файл [`main.rs`](/src/main.rs) - утримує всього одну функцію `fn main()` та є модулем, що можна запускати.  
До його задач входить підключення бібліотеки [`lib.rs`](/src/lib.rs)
```rust
use talys_empire::Core;
```
та вказівка щодо знаходження директорії з `.dat` файлами
```rust 
let core = Core::new("dat");
```
Після цього викликається функція `core.convert()?`, в якій зосереджена основна логіка переформатування даних.
## `lib.rs`
