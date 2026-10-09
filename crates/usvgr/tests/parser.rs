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

mod numbers_set_from_code {
    //! A number set from code (fframes' `svgr!` with an expression) is stored as
    //! `Float(value, "")`: it has no text. Readers must take the number, not the empty text,
    //! so it renders exactly as the same number written in the SVG (`Float(value, "value")`).

    use usvgr::svgtree::{
        AId, Attribute, EId, NestedNodeData, NestedNodeKind, NestedSvgDocument, StringStorage,
        SvgAttributeValue,
    };

    type Attrs = Vec<(AId, SvgAttributeValue<'static>)>;

    fn el(
        tag_name: EId,
        attrs: Attrs,
        children: Vec<NestedNodeData<'static>>,
    ) -> NestedNodeData<'static> {
        NestedNodeData {
            kind: NestedNodeKind::Element { tag_name },
            attrs: attrs
                .into_iter()
                .map(|(name, value)| Attribute { name, value })
                .collect(),
            children: children.into_iter().map(Some).collect(),
            static_hash: None,
        }
    }

    fn text(content: &'static str) -> NestedNodeData<'static> {
        NestedNodeData {
            kind: NestedNodeKind::Text(StringStorage::Borrowed(content)),
            attrs: Box::new([]),
            children: vec![],
            static_hash: None,
        }
    }

    fn s(value: &'static str) -> SvgAttributeValue<'static> {
        SvgAttributeValue::from(value)
    }

    fn svg(children: Vec<NestedNodeData<'static>>) -> NestedSvgDocument<'static> {
        NestedSvgDocument::from_nodes(vec![Some(el(
            EId::Svg,
            vec![(AId::Width, s("200")), (AId::Height, s("100"))],
            children,
        ))])
    }

    fn written(doc: &NestedSvgDocument) -> String {
        let mut fontdb = usvgr::fontdb::Database::new();
        fontdb
            .load_font_data(include_bytes!("../../svgr/tests/fonts/NotoSans-Regular.ttf").to_vec());
        usvgr::Tree::from_nested_svgtree(doc, &usvgr::Options::default(), &fontdb)
            .unwrap()
            .to_string(&usvgr::WriteOptions {
                preserve_text: true,
                ..Default::default()
            })
    }

    /// Builds `doc` with the number written in the SVG and set from code, and checks both
    /// render the same, and that the written one shows `expected`.
    fn same_as_written(
        number: f32,
        written_as: &'static str,
        expected: &str,
        doc: impl Fn(SvgAttributeValue<'static>) -> NestedSvgDocument<'static>,
    ) {
        let from_svg = written(&doc(SvgAttributeValue::Float(
            number,
            StringStorage::Borrowed(written_as),
        )));
        let from_code = written(&doc(SvgAttributeValue::from(number)));
        assert!(
            from_svg.contains(expected),
            "the case doesn't show {expected}:\n{from_svg}"
        );
        assert_eq!(from_code, from_svg);
    }

    fn filtered(primitive: NestedNodeData<'static>) -> NestedSvgDocument<'static> {
        svg(vec![
            el(
                EId::Defs,
                vec![],
                vec![el(EId::Filter, vec![(AId::Id, s("f"))], vec![primitive])],
            ),
            el(
                EId::Rect,
                vec![
                    (AId::Width, s("50")),
                    (AId::Height, s("50")),
                    (AId::Fill, s("red")),
                    (AId::Filter, s("url(#f)")),
                ],
                vec![],
            ),
        ])
    }

    fn label(attr: AId, value: SvgAttributeValue<'static>) -> NestedSvgDocument<'static> {
        svg(vec![el(
            EId::Text,
            vec![
                (AId::X, s("10")),
                (AId::Y, s("60")),
                (AId::FontFamily, s("Noto Sans")),
                (AId::FontSize, s("40")),
                (attr, value),
            ],
            vec![text("Ab")],
        )])
    }

    #[test]
    fn font_weight() {
        same_as_written(700.0, "700", r#"font-weight="700""#, |v| {
            label(AId::FontWeight, v)
        });
    }

    /// A parent `<text>` with `parent` weight and a `<tspan>` with `child` weight.
    fn nested_weights(
        parent: SvgAttributeValue<'static>,
        child: SvgAttributeValue<'static>,
    ) -> NestedSvgDocument<'static> {
        svg(vec![el(
            EId::Text,
            vec![
                (AId::X, s("10")),
                (AId::Y, s("60")),
                (AId::FontFamily, s("Noto Sans")),
                (AId::FontSize, s("40")),
                (AId::FontWeight, parent),
            ],
            vec![el(
                EId::Tspan,
                vec![(AId::FontWeight, child)],
                vec![text("Ab")],
            )],
        )])
    }

    #[test]
    fn font_weight_not_a_number_keeps_the_inherited_weight() {
        for invalid in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert_eq!(
                written(&nested_weights(s("700"), SvgAttributeValue::from(invalid))),
                written(&nested_weights(s("700"), s("invalid"))),
            );
        }
    }

    #[test]
    fn font_weight_lighter_than_a_small_number() {
        let doc = nested_weights(SvgAttributeValue::from(1.0), s("lighter"));
        assert!(written(&doc).contains(r#"font-weight="100""#));
    }

    #[test]
    fn text_rotate() {
        same_as_written(20.0, "20", r#"rotate="20"#, |v| label(AId::Rotate, v));
    }

    #[test]
    fn marker_orient() {
        same_as_written(45.0, "45", "0.7071068", |v| {
            svg(vec![
                el(
                    EId::Defs,
                    vec![],
                    vec![el(
                        EId::Marker,
                        vec![
                            (AId::Id, s("m")),
                            (AId::MarkerWidth, s("10")),
                            (AId::MarkerHeight, s("10")),
                            (AId::Orient, v),
                        ],
                        vec![el(
                            EId::Path,
                            vec![(AId::D, s("M0 0 L10 5 L0 10z"))],
                            vec![],
                        )],
                    )],
                ),
                el(
                    EId::Path,
                    vec![
                        (AId::D, s("M10 50 L90 50")),
                        (AId::Stroke, s("black")),
                        (AId::MarkerEnd, s("url(#m)")),
                    ],
                    vec![],
                ),
            ])
        });
    }

    #[test]
    fn color_matrix_values() {
        same_as_written(45.0, "45", r#"values="45""#, |v| {
            filtered(el(
                EId::FeColorMatrix,
                vec![(AId::Type, s("hueRotate")), (AId::Values, v)],
                vec![],
            ))
        });
    }

    #[test]
    fn morphology_radius() {
        same_as_written(2.0, "2", r#"radius="2 2""#, |v| {
            filtered(el(
                EId::FeMorphology,
                vec![(AId::Operator, s("dilate")), (AId::Radius, v)],
                vec![],
            ))
        });
    }

    #[test]
    fn turbulence_base_frequency() {
        same_as_written(0.5, "0.5", r#"baseFrequency="0.5 0.5""#, |v| {
            filtered(el(EId::FeTurbulence, vec![(AId::BaseFrequency, v)], vec![]))
        });
    }

    #[test]
    fn transfer_table_values() {
        same_as_written(0.5, "0.5", r#"tableValues="0.5""#, |v| {
            filtered(el(
                EId::FeComponentTransfer,
                vec![],
                vec![el(
                    EId::FeFuncR,
                    vec![(AId::Type, s("discrete")), (AId::TableValues, v)],
                    vec![],
                )],
            ))
        });
    }
}
