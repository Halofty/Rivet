use rivet::routes::Route;

#[test]
fn typed_links_round_trip_with_entity_ids() {
    for route in [
        Route::Home {},
        Route::Projects {},
        Route::ProjectDetail { project_id: 2 },
        Route::IssueDetail { issue_id: 6 },
        Route::Connection {},
    ] {
        assert_eq!(route.to_string().parse::<Route>().unwrap(), route);
    }
}

#[test]
fn malformed_and_unknown_paths_reach_the_fallback() {
    for path in [
        "/unknown",
        "/projects/not-a-number",
        "/issues/not-a-number",
        "/projects/1/extra",
    ] {
        assert!(
            matches!(path.parse::<Route>().unwrap(), Route::NotFound { .. }),
            "{path}"
        );
    }
}
