#[test]
fn clippath_with_invalid_child() {
    let svg = "
    <svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 1 1'>
        <clipPath id='clip1'>
            <rect/>
        </clipPath>
        <rect clip-path='url(#clip1)' width='10' height='10'/>
    </svg>
    ";

    let fontdb = usvgr::fontdb::Database::new();
    let tree = usvgr::Tree::from_str(&svg, &usvgr::Options::default(), &fontdb).unwrap();
    // clipPath is invalid and should be removed together with rect.
    assert_eq!(tree.root().has_children(), false);
}

#[test]
fn simplify_paths() {
    let svg = "
    <svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 1 1'>
        <path d='M 10 20 L 10 30 Z Z Z'/>
    </svg>
    ";

    let fontdb = usvgr::fontdb::Database::new();
    let tree = usvgr::Tree::from_str(&svg, &usvgr::Options::default(), &fontdb).unwrap();
    let path = &tree.root().children()[0];
    match path {
        usvgr::Node::Path(ref path) => {
            // Make sure we have MLZ and not MLZZZ
            assert_eq!(path.data().verbs().len(), 3);
        }
        _ => unreachable!(),
    };
}

#[test]
fn size_detection_1() {
    let svg = "<svg viewBox='0 0 10 20' xmlns='http://www.w3.org/2000/svg'/>";
    let fontdb = usvgr::fontdb::Database::new();
    let tree = usvgr::Tree::from_str(&svg, &usvgr::Options::default(), &fontdb).unwrap();
    assert_eq!(tree.size(), usvgr::Size::from_wh(10.0, 20.0).unwrap());
}

#[test]
fn size_detection_2() {
    let svg =
        "<svg width='30' height='40' viewBox='0 0 10 20' xmlns='http://www.w3.org/2000/svg'/>";
    let fontdb = usvgr::fontdb::Database::new();
    let tree = usvgr::Tree::from_str(&svg, &usvgr::Options::default(), &fontdb).unwrap();
    assert_eq!(tree.size(), usvgr::Size::from_wh(30.0, 40.0).unwrap());
}

#[test]
fn size_detection_3() {
    let svg =
        "<svg width='50%' height='100%' viewBox='0 0 10 20' xmlns='http://www.w3.org/2000/svg'/>";
    let fontdb = usvgr::fontdb::Database::new();
    let tree = usvgr::Tree::from_str(&svg, &usvgr::Options::default(), &fontdb).unwrap();
    assert_eq!(tree.size(), usvgr::Size::from_wh(5.0, 20.0).unwrap());
}

#[test]
fn size_detection_4() {
    let svg = "
    <svg xmlns='http://www.w3.org/2000/svg'>
        <circle cx='18' cy='18' r='18'/>
    </svg>
    ";
    let fontdb = usvgr::fontdb::Database::new();
    let tree = usvgr::Tree::from_str(&svg, &usvgr::Options::default(), &fontdb).unwrap();
    assert_eq!(tree.size(), usvgr::Size::from_wh(36.0, 36.0).unwrap());
    assert_eq!(
        tree.view_box().rect,
        usvgr::NonZeroRect::from_xywh(0.0, 0.0, 36.0, 36.0).unwrap()
    );
}

#[test]
fn size_detection_5() {
    let svg = "<svg xmlns='http://www.w3.org/2000/svg'/>";
    let fontdb = usvgr::fontdb::Database::new();
    let tree = usvgr::Tree::from_str(&svg, &usvgr::Options::default(), &fontdb).unwrap();
    assert_eq!(tree.size(), usvgr::Size::from_wh(100.0, 100.0).unwrap());
}

#[test]
fn invalid_size_1() {
    let svg = "<svg width='0' height='0' viewBox='0 0 10 20' xmlns='http://www.w3.org/2000/svg'/>";
    let fontdb = usvgr::fontdb::Database::new();
    let result = usvgr::Tree::from_str(&svg, &usvgr::Options::default(), &fontdb);
    assert!(result.is_err());
}

#[test]
fn tree_is_send_and_sync() {
    fn ensure_send_and_sync<T: Send + Sync>() {}
    ensure_send_and_sync::<usvgr::Tree>();
}

#[test]
fn keeps_data_fframes_attributes_on_nodes() {
    let svg = "
    <svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 10 10'>
        <g data-fframes-inspect='allow-offcanvas' data-fframes-cache='static'>
            <rect width='10' height='10' data-fframes-inspect='a b'/>
        </g>
        <path d='M 0 0 L 10 10'/>
        <defs><rect id='r' width='2' height='2' data-fframes-tag='shared'/></defs>
        <use href='#r' data-fframes-tag='use'/>
    </svg>
    ";

    let fontdb = usvgr::fontdb::Database::new();
    let tree = usvgr::Tree::from_str(&svg, &usvgr::Options::default(), &fontdb).unwrap();
    let children = tree.root().children();

    let usvgr::Node::Group(group) = &children[0] else {
        panic!("group")
    };
    assert_eq!(group.fframes_data().get("inspect"), Some("allow-offcanvas"));
    assert_eq!(group.fframes_data().get("cache"), Some("static"));
    assert_eq!(
        group.fframes_data().iter().collect::<Vec<_>>(),
        vec![("inspect", "allow-offcanvas"), ("cache", "static")]
    );
    assert_eq!(
        group.children()[0].fframes_data().get("inspect"),
        Some("a b")
    );

    assert!(children[1].fframes_data().is_empty());

    let usvgr::Node::Group(use_group) = &children[2] else {
        panic!("use")
    };
    assert_eq!(use_group.fframes_data().get("tag"), Some("use"));
    assert_eq!(
        use_group.children()[0].fframes_data().get("tag"),
        Some("shared")
    );

    let written = tree.to_string(&usvgr::WriteOptions::default());
    assert!(
        written.contains(r#"data-fframes-inspect="allow-offcanvas""#),
        "{written}"
    );
}

#[test]
fn keeps_data_fframes_attributes_on_text() {
    let svg = "
    <svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 100 100'>
        <text x='10' y='50' font-family='Noto Sans' data-fframes-inspect='allow-offcanvas'>a</text>
        <text x='10' y='50' font-family='Noto Sans'>a</text>
    </svg>
    ";

    let mut fontdb = usvgr::fontdb::Database::new();
    fontdb.load_fonts_dir("../svgr/tests/fonts");
    let options = usvgr::Options::default();
    // The same text twice: the second must not get the first one's data from the outline cache.
    let tree = usvgr::Tree::from_str(&svg, &options, &fontdb).unwrap();
    let texts: Vec<_> = tree
        .root()
        .children()
        .iter()
        .filter_map(|node| match node {
            usvgr::Node::Text(text) => Some(text),
            _ => None,
        })
        .collect();

    assert_eq!(texts.len(), 2);
    assert_eq!(
        texts[0].fframes_data().get("inspect"),
        Some("allow-offcanvas")
    );
    assert_eq!(
        texts[0].flattened().fframes_data().get("inspect"),
        Some("allow-offcanvas")
    );
    assert!(texts[1].fframes_data().is_empty());
    assert!(texts[1].flattened().fframes_data().is_empty());
}
