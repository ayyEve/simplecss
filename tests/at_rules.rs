// Copyright 2025 the SimpleCSS Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! At Rules
#[cfg(feature="at_rules")]
use simplecss::*;
#[cfg(feature="at_rules")]
use simplecss::at_rules::*;
#[cfg(feature="at_rules")]
use simplecss::at_rules::at_rule::*;


#[test]
#[cfg(feature="at_rules")]
fn container() {
    use at_rule::Comparison;
    use container::*;
    // tests are examples from https://developer.mozilla.org/en-US/docs/Web/CSS/@container
    let style = StyleSheet::parse(
        r#"
        /* 1. With a <size-query> */
        @container (width > 400px) {
            div { color: red; }
        }

        /* 2. With an optional <container-name> */
        @container tall (height > 30rem) {
            div { color: red; }
        }

        /* 3. With a <scroll-state> */
        @container scroll-state(scrollable: top) {
            div { color: red; }
        }

        /* 4. With a <container-name> and a <scroll-state> */
        @container sticky-heading scroll-state(stuck: top) {
            div { color: red; }
        }

        /* 5. Multiple queries in a single condition */
        @container (width > 400px) and style(--responsive: true) {
            div { color: red; }
        }

        /* 6. Condition list */
        @container card (width > 400px), style(--responsive: true), scroll-state(stuck: top) {
            div { color: red; }
        }

        a { color:red }"#,
    );
    assert_eq!(style.to_string(), "a { color:red; }");
    
    let rule = Rule {
        selector: Selector::parse("div").unwrap(),
        declarations: vec![
            Declaration {
                name: "color",
                value: "red",
                important: false
            }
        ]
    };

    let rule1 = AtRule::Container(Container {
        conditions: vec![
            ContainerCondition::QueryOnly(ContainerQuery::List { 
                first: ContainerQueryInParens::Feature(Feature {
                    key: "width",
                    comparison: Comparison::Greater,
                    value: "400px",
                }), 
                rest: vec![] 
            })
        ],
        contents: StyleSheet { 
            rules: vec![ rule.clone() ], 
            at_rules: vec![]
        },
    });
    let rule2 = AtRule::Container(Container {
        conditions: vec![
            ContainerCondition::NameAndQuery {
                name: "tall",
                query: ContainerQuery::List { 
                    first: ContainerQueryInParens::Feature(Feature {
                        key: "height",
                        comparison: Comparison::Greater,
                        value: "30rem",
                    }), 
                    rest: vec![] 
                }
            }
        ],
        contents: StyleSheet { 
            rules: vec![ rule.clone() ], 
            at_rules: vec![]
        },
    });
    let rule3 = AtRule::Container(Container {
        conditions: vec![
            ContainerCondition::QueryOnly(ContainerQuery::List { 
                first: ContainerQueryInParens::Function(ContainerFunction {
                    name: "scroll-state",
                    query: FunctionQuery::List { 
                        first: FunctionInParens::Feature(Feature {
                            key: "scrollable",
                            comparison: Comparison::Equal,
                            value: "top",
                        }), 
                        rest: vec![] 
                    }
                }), 
                rest: vec![] 
            })
        ],
        contents: StyleSheet { 
            rules: vec![ rule.clone() ], 
            at_rules: vec![]
        },
    });
    let rule4 = AtRule::Container(Container {
        conditions: vec![
            ContainerCondition::NameAndQuery {
                name: "sticky-heading",
                query: ContainerQuery::List { 
                    first: ContainerQueryInParens::Function(ContainerFunction {
                        name: "scroll-state",
                        query: FunctionQuery::List { 
                            first: FunctionInParens::Feature(Feature {
                                key: "stuck",
                                comparison: Comparison::Equal,
                                value: "top",
                            }), 
                            rest: vec![] 
                        }
                    }), 
                    rest: vec![] 
                }
            }
        ],
        contents: StyleSheet { 
            rules: vec![ rule.clone() ], 
            at_rules: vec![]
        },
    });
    let rule5 = AtRule::Container(Container {
        conditions: vec![
            ContainerCondition::QueryOnly(
                ContainerQuery::List { 
                    first: ContainerQueryInParens::Feature(Feature {
                        key: "width", 
                        comparison: Comparison::Greater, 
                        value: "400px",
                    }),  
                    rest: vec![
                        ContainerQueryAndOr::And(ContainerQueryInParens::Function(ContainerFunction { 
                            name: "style", 
                            query: FunctionQuery::List { 
                                first: FunctionInParens::Feature(Feature {
                                    key: "--responsive",
                                    comparison: Comparison::Equal,
                                    value: "true",
                                }), 
                                rest: vec![] 
                            }
                        }))
                    ] 
                }
            )
        ],
        contents: StyleSheet { 
            rules: vec![ rule.clone() ], 
            at_rules: vec![]
        },
    });
    let rule6 = AtRule::Container(Container {
        conditions: vec![
            ContainerCondition::NameAndQuery{
                name: "card",
                query: ContainerQuery::List { 
                    first: ContainerQueryInParens::Feature(Feature {
                        key: "width", 
                        comparison: Comparison::Greater, 
                        value: "400px",
                    }),
                    rest: vec![] ,
                }
            },
            
            ContainerCondition::QueryOnly(
                ContainerQuery::List { 
                    first: ContainerQueryInParens::Function(ContainerFunction {
                        name: "style",
                        query: FunctionQuery::List { 
                            first: FunctionInParens::Feature(Feature {
                                key: "--responsive",
                                comparison: Comparison::Equal,
                                value: "true",
                            }), 
                            rest: vec![]
                        }
                    }),
                    rest: vec![],
                }
            ),

            ContainerCondition::QueryOnly(
                ContainerQuery::List { 
                    first: ContainerQueryInParens::Function(ContainerFunction {
                        name: "scroll-state",
                        query: FunctionQuery::List { 
                            first: FunctionInParens::Feature(Feature {
                                key: "stuck",
                                comparison: Comparison::Equal,
                                value: "top",
                            }), 
                            rest: vec![]
                        }
                    }),
                    rest: vec![],
                }
            ),
        ],
        contents: StyleSheet { 
            rules: vec![ rule.clone() ], 
            at_rules: vec![]
        },
    });

    let rules = [
        rule1,
        rule2,
        rule3,
        rule4,
        rule5,
        rule6,
    ];
    
    assert_rules(style, rules);
}

#[test]
#[cfg(feature="at_rules")]
fn font_face() {
    // tests are examples from https://developer.mozilla.org/en-US/docs/Web/CSS/@container
    let style = StyleSheet::parse(
        r#"
        @font-face {
            font-family: "Trickster";
            src: local("Trickster"),
    url("trickster-COLRv1.otf") format("opentype") tech(color-COLRv1),
    url("trickster-outline.otf") format("opentype"),
    url("trickster-outline.woff") format("woff");
        }
        a { color:red }"#,
    );

    let rule1 = AtRule::FontFace(vec![
        Declaration {
            name: "font-family",
            value: "\"Trickster\"",
            important: false,
        },
        Declaration {
            name: "src",
            value: r#"local("Trickster"),
    url("trickster-COLRv1.otf") format("opentype") tech(color-COLRv1),
    url("trickster-outline.otf") format("opentype"),
    url("trickster-outline.woff") format("woff")"#,
            important: false,
        },
    ]);

    assert_eq!(style.to_string(), "a { color:red; }");

    assert_eq!(style.at_rules, vec![
        rule1,
    ]);
}


#[test]
#[cfg(feature="at_rules")]
fn import() {
    use crate::media::*;
    use crate::import::*;

    let style = StyleSheet::parse(
        r#"
        @import "custom.css";
        @import url("chrome://communicator/skin/");
        @import src("some-source");

        /* layer tests */
        @import "test" layer;
        @import "test" layer(test-layer);

        /* media query tests */
        @import url("fine-print.css") print;
        @import src("bluish.css") print, screen;
        @import "common.css" screen;
        @import url("landscape.css") screen and (orientation: landscape);

        a { color:red }"#,
    );
    assert_eq!(style.to_string(), "a { color:red; }");

    let rule1 = AtRule::Import(Import {
        url: ImportUrl::String("custom.css"),
        layer: None,
        supports: None,
        media_queries: vec![]
    });
    let rule2 = AtRule::Import(Import {
        url: ImportUrl::Url("chrome://communicator/skin/"),
        layer: None,
        supports: None,
        media_queries: vec![]
    });
    let rule3 = AtRule::Import(Import {
        url: ImportUrl::Src("some-source"),
        layer: None,
        supports: None,
        media_queries: vec![]
    });

    // layer tests
    let layer1 = AtRule::Import(Import {
        url: ImportUrl::String("test"),
        layer: Some(ImportLayer::Layer),
        supports: None,
        media_queries: vec![]
    });
    let layer2 = AtRule::Import(Import {
        url: ImportUrl::String("test"),
        layer: Some(ImportLayer::Named("test-layer")),
        supports: None,
        media_queries: vec![]
    });
    
    // media query tests
    let mq1 = AtRule::Import(Import {
        url: ImportUrl::Url("fine-print.css"),
        layer: None,
        supports: None,
        media_queries: vec![
            MediaQuery::OtherThing {
                not_only: None,
                media_type: "print",
                conditions: vec![],
            }
        ]
    });

    let mq2 = AtRule::Import(Import {
        url: ImportUrl::Src("bluish.css"),
        layer: None,
        supports: None,
        media_queries: vec![
            MediaQuery::OtherThing {
                not_only: None,
                media_type: "print",
                conditions: vec![],
            },
            MediaQuery::OtherThing {
                not_only: None,
                media_type: "screen",
                conditions: vec![],
            }
        ]
    });

    let mq3 = AtRule::Import(Import {
        url: ImportUrl::String("common.css"),
        layer: None,
        supports: None,
        media_queries: vec![
            MediaQuery::OtherThing {
                not_only: None,
                media_type: "screen",
                conditions: vec![],
            }
        ]
    });

    let mq4 = AtRule::Import(Import {
        url: ImportUrl::Url("landscape.css"),
        layer: None,
        supports: None,
        media_queries: vec![

            MediaQuery::OtherThing {
                not_only: None,
                media_type: "screen",
                conditions: vec![
                    MediaConditionWithoutOr::Media {
                        media: MediaInParens::Feature(MediaFeature::KeyVal { 
                            key: "orientation", 
                            val: "landscape",
                        }),
                        conditions: vec![]
                    }
                ]
            }
            
        ]
    });

    let rules = [
        rule1,
        rule2,
        rule3,

        layer1,
        layer2,

        mq1,
        mq2,
        mq3,
        mq4
    ];

    assert_rules(style, rules);
}


#[test]
#[cfg(feature="at_rules")]
fn keyframes() {
    use at_rule::KeyFrame;
    let style = StyleSheet::parse(
        r#"
        @keyframes test-anim {
            from { color: red; }
            50% { color: cyan; }
            to { color: blue; }
        }
        a { color:red }"#,
    );
    assert_eq!(style.to_string(), "a { color:red; }");

    let rule = AtRule::Keyframes { 
        name: "test-anim", 
        frames: vec![
            KeyFrame {
                key: "from",
                declarations: vec![
                    Declaration {
                        name: "color",
                        value: "red",
                        important: false,
                    }
                ]
            },
            KeyFrame {
                key: "50%",
                declarations: vec![
                    Declaration {
                        name: "color",
                        value: "cyan",
                        important: false,
                    }
                ]
            },
            KeyFrame {
                key: "to",
                declarations: vec![
                    Declaration {
                        name: "color",
                        value: "blue",
                        important: false,
                    }
                ]
            },
        ]
    };
    println!("{style:?}");
    assert_eq!(style.at_rules[0], rule);
}


#[test]
#[cfg(feature="at_rules")]
fn layer() {
    use at_rule::LayerType;
    let style = StyleSheet::parse(
        r#"
        @layer module, state;

        @layer state {
            div { color: red; }
        }

        @layer module {
            div { color: red; }
        } 
        a { color:red }"#,
    );
    assert_eq!(style.to_string(), "a { color:red; }");
    
    let rule = Rule {
        selector: Selector::parse("div").unwrap(),
        declarations: vec![
            Declaration {
                name: "color",
                value: "red",
                important: false
            }
        ]
    };

    let rule1 = AtRule::Layer(LayerType::Statement(vec!["module", "state"]));
    let rule2 = AtRule::Layer(LayerType::Block {
        name: Some("state"),
        rules: vec![ rule.clone() ]
    });
    let rule3 = AtRule::Layer(LayerType::Block {
        name: Some("module"),
        rules: vec![ rule.clone() ]
    });

    println!("{style:?}");
    assert_eq!(style.at_rules, vec![
        rule1,
        rule2,
        rule3,
    ]);
}


#[test]
#[cfg(feature="at_rules")]
fn media() {
    use media::*;
    let style = StyleSheet::parse(
        r#"
        @media screen {
            div { color: red; }
        }

        @media only screen and (orientation: landscape) {
            div { color: red; }
        }

        /* When the width is between 600px and 900px OR above 1100px - change the appearance of <div> */
        @media screen and (max-width: 900px) and (min-width: 600px), (min-width: 1100px) {
            div { color: red; }
        }

        a { color:red }"#,
    ); 
    let rule = Rule {
        selector: Selector::parse("div").unwrap(),
        declarations: vec![
            Declaration {
                name: "color",
                value: "red",
                important: false
            }
        ]
    };

    let rule1 = AtRule::Media(Media {
        rules: vec![ rule.clone() ],
        query: vec![
            MediaQuery::OtherThing {
                not_only: None,
                media_type: "screen",
                conditions: Vec::new()
            },
        ],
    });
    let rule2 = AtRule::Media(Media {
        rules: vec![ rule.clone() ],
        query: vec![
            MediaQuery::OtherThing {
                not_only: Some(MediaNotOnly::Only),
                media_type: "screen",
                conditions: vec![
                    MediaConditionWithoutOr::Media {
                        media: MediaInParens::Feature(MediaFeature::KeyVal { key: "orientation", val: "landscape" }),
                        conditions: vec![]
                    }
                ],
            },
        ],
    });

    let rule3 = AtRule::Media(Media {
        rules: vec![ rule.clone() ],
        query: vec![
            MediaQuery::OtherThing {
                not_only: None,
                media_type: "screen",
                conditions: vec![
                    MediaConditionWithoutOr::Media {
                        media: MediaInParens::Feature(MediaFeature::KeyVal { 
                            key: "max-width", 
                            val: "900px" 
                        }),
                        conditions: vec![
                            MediaAnd(MediaInParens::Feature(MediaFeature::KeyVal { 
                                key: "min-width", 
                                val: "600px"
                            }))
                        ]
                    }
                ],
            },
            MediaQuery::Condition(MediaCondition::List { 
                first: Box::new(MediaInParens::Feature(MediaFeature::KeyVal { 
                    key: "min-width", 
                    val: "1100px"
                })), 
                conditions: vec![]
            })
        ],
    });


    let rules = [
        rule1,
        rule2,
        rule3,
    ];
    assert_rules(style, rules);
}

#[test]
#[cfg(feature="at_rules")]
fn namespace() {
    // https://developer.mozilla.org/en-US/docs/Web/CSS/@namespace
    let style = StyleSheet::parse(
        r#"
        /* Default namespace */
        @namespace url(XML-namespace-URL);
        @namespace "XML-namespace-URL";

        /* Prefixed namespace */
        @namespace prefix url(XML-namespace-URL);
        @namespace prefix "XML-namespace-URL";

        a { color:red }"#,
    );
    assert_eq!(style.to_string(), "a { color:red; }");
    
    let rule1 = AtRule::Namespace { 
        name: None, 
        value: "url(XML-namespace-URL)",
    };
    let rule2 = AtRule::Namespace { 
        name: None, 
        value: "XML-namespace-URL",
    };
    let rule3 = AtRule::Namespace { 
        name: Some("prefix"), 
        value: "url(XML-namespace-URL)",
    };
    let rule4 = AtRule::Namespace { 
        name: Some("prefix"), 
        value: "\"XML-namespace-URL\"",
    };

    assert_eq!(style.at_rules, vec![
        rule1,
        rule2,
        rule3,
        rule4,
    ]);
}



#[test]
#[cfg(feature="at_rules")]
fn supports() {
    use crate::supports::*;

    let style = StyleSheet::parse(
        r#"
        @supports not (not (transform-origin: 2px)) {
            div { color: red; }
        }
        @supports (display: grid) and (not (display: inline-grid)) {
            div { color: red; }
        }

        @supports (animation-name: test) {
            div { color: red; }
        }
        a { color:red }"#,
    );
    assert_eq!(style.to_string(), "a { color:red; }");
    
    let rule = Rule {
        selector: Selector::parse("div").unwrap(),
        declarations: vec![
            Declaration {
                name: "color",
                value: "red",
                important: false
            }
        ]
    };

    let rule1 = AtRule::Supports(Supports { 
        condition: SupportsCondition::Not(
            SupportsInParens::Condition(Box::new(
                SupportsCondition::Not(
                    SupportsInParens::Feature(Declaration { 
                        name: "transform-origin", 
                        value: "2px", 
                        important: false 
                    })
                )
            ))
        ),
        rules: vec![ rule.clone() ]  
    });
    let rule2 = AtRule::Supports(Supports { 
        condition: SupportsCondition::List { 
            first: SupportsInParens::Feature(Declaration { name: "display", value: "grid", important: false }), 
            list: vec![
                SupportsAndOr::And(SupportsInParens::Condition(Box::new(SupportsCondition::Not(
                    SupportsInParens::Feature(Declaration { name: "display", value: "inline-grid", important: false })
                ))))
            ]
        },
        rules: vec![ rule.clone() ]  
    });

    let rules = [
        rule1,
        rule2,
    ];

    assert_rules(style, rules);
}



#[cfg(feature="at_rules")]
fn assert_rules<const N:usize>(style: StyleSheet<'_>, rules: [AtRule<'_>; N]) {
    #[allow(clippy::needless_range_loop, reason = "indexing two things with i, cleaner code than suggestion")]
    for i in 0..rules.len() {
        assert_eq!(
            style.at_rules[i],
            rules[i],
            "at rule {} did not parse correctly", i+1
        );
    }
}
