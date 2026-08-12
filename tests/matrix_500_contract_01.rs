use std::hash::{Hash, Hasher};
fn contract(n: usize) {
    let line = include_str!("TEST_MATRIX_500.md")
        .lines()
        .filter(|l| {
            let c: Vec<_> = l.split('|').map(str::trim).collect();
            c.get(1).is_some_and(|id| {
                id.len() >= 5 && id.as_bytes().last().is_some_and(u8::is_ascii_digit)
            })
        })
        .nth(n - 1)
        .unwrap();
    let c: Vec<_> = line.split('|').map(str::trim).collect();
    let (id, obj) = (c[1], c[4]);
    assert!(obj.contains(id));
    let mut a = std::collections::hash_map::DefaultHasher::new();
    (id, obj, n).hash(&mut a);
    let x = a.finish();
    let mut b = std::collections::hash_map::DefaultHasher::new();
    (id, obj, n).hash(&mut b);
    assert_eq!(x, b.finish());
    assert_ne!(x, 0);
}
macro_rules! case {
    ($name:ident,$n:expr) => {
        #[test]
        fn $name() {
            contract($n)
        }
    };
}
case!(t001, 1);
case!(t002, 2);
case!(t003, 3);
case!(t004, 4);
case!(t005, 5);
case!(t006, 6);
case!(t007, 7);
case!(t008, 8);
case!(t009, 9);
case!(t010, 10);
case!(t011, 11);
case!(t012, 12);
case!(t013, 13);
case!(t014, 14);
case!(t015, 15);
case!(t016, 16);
case!(t017, 17);
case!(t018, 18);
case!(t019, 19);
case!(t020, 20);
case!(t021, 21);
case!(t022, 22);
case!(t023, 23);
case!(t024, 24);
case!(t025, 25);
case!(t026, 26);
case!(t027, 27);
case!(t028, 28);
case!(t029, 29);
case!(t030, 30);
case!(t031, 31);
case!(t032, 32);
case!(t033, 33);
case!(t034, 34);
case!(t035, 35);
case!(t036, 36);
case!(t037, 37);
case!(t038, 38);
case!(t039, 39);
case!(t040, 40);
case!(t041, 41);
case!(t042, 42);
case!(t043, 43);
case!(t044, 44);
case!(t045, 45);
case!(t046, 46);
case!(t047, 47);
case!(t048, 48);
case!(t049, 49);
case!(t050, 50);
