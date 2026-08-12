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
    let len = c[4].chars().count();
    assert!(len > c[1].len());
    assert!((1..=500).contains(&n));
    assert_eq!(c[4].matches(c[1]).count(), 1);
}
macro_rules! case {
    ($i:ident,$n:expr) => {
        #[test]
        fn $i() {
            contract($n)
        }
    };
}
case!(t301, 301);
case!(t302, 302);
case!(t303, 303);
case!(t304, 304);
case!(t305, 305);
case!(t306, 306);
case!(t307, 307);
case!(t308, 308);
case!(t309, 309);
case!(t310, 310);
case!(t311, 311);
case!(t312, 312);
case!(t313, 313);
case!(t314, 314);
case!(t315, 315);
case!(t316, 316);
case!(t317, 317);
case!(t318, 318);
case!(t319, 319);
case!(t320, 320);
case!(t321, 321);
case!(t322, 322);
case!(t323, 323);
case!(t324, 324);
case!(t325, 325);
case!(t326, 326);
case!(t327, 327);
case!(t328, 328);
case!(t329, 329);
case!(t330, 330);
case!(t331, 331);
case!(t332, 332);
case!(t333, 333);
case!(t334, 334);
case!(t335, 335);
case!(t336, 336);
case!(t337, 337);
case!(t338, 338);
case!(t339, 339);
case!(t340, 340);
case!(t341, 341);
case!(t342, 342);
case!(t343, 343);
case!(t344, 344);
case!(t345, 345);
case!(t346, 346);
case!(t347, 347);
case!(t348, 348);
case!(t349, 349);
case!(t350, 350);
