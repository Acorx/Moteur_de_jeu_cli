use std::collections::HashSet;

fn matrix_rows() -> Vec<(&'static str, &'static str)> {
    include_str!("TEST_MATRIX_500.md")
        .lines()
        .filter_map(|line| {
            let cells: Vec<_> = line.split('|').map(str::trim).collect();
            let id = *cells.get(1)?;
            let objective = *cells.get(4)?;
            let valid = id.len() >= 5 && id.as_bytes().last().is_some_and(u8::is_ascii_digit);
            valid.then_some((id, objective))
        })
        .collect()
}

#[test]
fn matrix_manifest_has_exactly_500_unique_objectives() {
    let rows = matrix_rows();
    assert_eq!(
        rows.len(),
        500,
        "la matrice doit contenir exactement 500 cas"
    );
    let ids: HashSet<_> = rows.iter().map(|(id, _)| *id).collect();
    assert_eq!(
        ids.len(),
        500,
        "chaque identifiant de matrice doit être unique"
    );
    assert!(rows.iter().all(|(id, objective)| objective.contains(id)));
}
