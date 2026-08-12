fn contract(n: usize) {
    let rows: Vec<_> = include_str!("TEST_MATRIX_500.md")
        .lines()
        .filter(|l| {
            l.starts_with('|')
                && l.split('|')
                    .nth(1)
                    .is_some_and(|x| x.trim().as_bytes().last().is_some_and(u8::is_ascii_digit))
        })
        .collect();
    let row = rows[n - 1];
    let c: Vec<_> = row.split('|').map(str::trim).collect();
    assert_eq!(
        rows.iter()
            .filter(|r| r.split('|').nth(1).is_some_and(|x| x.trim() == c[1]))
            .count(),
        1
    );
    assert!(c[2].len() > 2);
    assert!(c[4].contains(c[1]));
}
macro_rules! case {
    ($i:ident,$n:expr) => {
        #[test]
        fn $i() {
            contract($n)
        }
    };
}
case!(t451, 451);
case!(t452, 452);
case!(t453, 453);
case!(t454, 454);
case!(t455, 455);
case!(t456, 456);
case!(t457, 457);
case!(t458, 458);
case!(t459, 459);
case!(t460, 460);
case!(t461, 461);
case!(t462, 462);
case!(t463, 463);
case!(t464, 464);
case!(t465, 465);
case!(t466, 466);
case!(t467, 467);
case!(t468, 468);
case!(t469, 469);
case!(t470, 470);
case!(t471, 471);
case!(t472, 472);
case!(t473, 473);
case!(t474, 474);
case!(t475, 475);
case!(t476, 476);
case!(t477, 477);
case!(t478, 478);
case!(t479, 479);
case!(t480, 480);
case!(t481, 481);
case!(t482, 482);
case!(t483, 483);
case!(t484, 484);
case!(t485, 485);
case!(t486, 486);
case!(t487, 487);
case!(t488, 488);
case!(t489, 489);
case!(t490, 490);
case!(t491, 491);
case!(t492, 492);
case!(t493, 493);
case!(t494, 494);
case!(t495, 495);
case!(t496, 496);
case!(t497, 497);
case!(t498, 498);
case!(t499, 499);
case!(t500, 500);
