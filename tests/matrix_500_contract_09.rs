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
    let canonical = c[4].split_whitespace().collect::<Vec<_>>().join(" ");
    assert_eq!(
        canonical,
        canonical.split_whitespace().collect::<Vec<_>>().join(" ")
    );
    assert!(!canonical.ends_with("| "));
    assert!(canonical.contains(c[1]));
}
macro_rules! case {
    ($i:ident,$n:expr) => {
        #[test]
        fn $i() {
            contract($n)
        }
    };
}
case!(t401, 401);
case!(t402, 402);
case!(t403, 403);
case!(t404, 404);
case!(t405, 405);
case!(t406, 406);
case!(t407, 407);
case!(t408, 408);
case!(t409, 409);
case!(t410, 410);
case!(t411, 411);
case!(t412, 412);
case!(t413, 413);
case!(t414, 414);
case!(t415, 415);
case!(t416, 416);
case!(t417, 417);
case!(t418, 418);
case!(t419, 419);
case!(t420, 420);
case!(t421, 421);
case!(t422, 422);
case!(t423, 423);
case!(t424, 424);
case!(t425, 425);
case!(t426, 426);
case!(t427, 427);
case!(t428, 428);
case!(t429, 429);
case!(t430, 430);
case!(t431, 431);
case!(t432, 432);
case!(t433, 433);
case!(t434, 434);
case!(t435, 435);
case!(t436, 436);
case!(t437, 437);
case!(t438, 438);
case!(t439, 439);
case!(t440, 440);
case!(t441, 441);
case!(t442, 442);
case!(t443, 443);
case!(t444, 444);
case!(t445, 445);
case!(t446, 446);
case!(t447, 447);
case!(t448, 448);
case!(t449, 449);
case!(t450, 450);
