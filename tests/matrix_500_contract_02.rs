fn contract(n: usize) {
    let row = include_str!("TEST_MATRIX_500.md")
        .lines()
        .filter(|l| {
            l.starts_with('|')
                && l.split('|')
                    .nth(1)
                    .is_some_and(|x| x.trim().as_bytes().last().is_some_and(u8::is_ascii_digit))
        })
        .nth(n - 1)
        .unwrap();
    let c: Vec<_> = row.split('|').map(str::trim).collect();
    let id = c[1];
    let parts: Vec<_> = id.split('-').collect();
    assert_eq!(parts.len(), 2);
    assert_eq!(parts[1].len(), 3);
    assert!(parts[1].parse::<u16>().is_ok());
    assert!(c[4].contains(id));
}
macro_rules! case {
    ($i:ident,$n:expr) => {
        #[test]
        fn $i() {
            contract($n)
        }
    };
}
case!(t051, 51);
case!(t052, 52);
case!(t053, 53);
case!(t054, 54);
case!(t055, 55);
case!(t056, 56);
case!(t057, 57);
case!(t058, 58);
case!(t059, 59);
case!(t060, 60);
case!(t061, 61);
case!(t062, 62);
case!(t063, 63);
case!(t064, 64);
case!(t065, 65);
case!(t066, 66);
case!(t067, 67);
case!(t068, 68);
case!(t069, 69);
case!(t070, 70);
case!(t071, 71);
case!(t072, 72);
case!(t073, 73);
case!(t074, 74);
case!(t075, 75);
case!(t076, 76);
case!(t077, 77);
case!(t078, 78);
case!(t079, 79);
case!(t080, 80);
case!(t081, 81);
case!(t082, 82);
case!(t083, 83);
case!(t084, 84);
case!(t085, 85);
case!(t086, 86);
case!(t087, 87);
case!(t088, 88);
case!(t089, 89);
case!(t090, 90);
case!(t091, 91);
case!(t092, 92);
case!(t093, 93);
case!(t094, 94);
case!(t095, 95);
case!(t096, 96);
case!(t097, 97);
case!(t098, 98);
case!(t099, 99);
case!(t100, 100);
