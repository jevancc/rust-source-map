extern crate source_map;

#[cfg(test)]
mod to_string_with_source_map {
    use source_map::*;
    use std::rc::Rc;

    #[test]
    fn merging_duplicate_mappings() {
        let mut input = SourceNode::new(None, None, None, None);
        add_sources_by_params(
            &mut input,
            &[
                (1, 0, "a.js", "(function", None),
                (1, 0, "a.js", "() {\n", None),
                (-1, -1, "", "  ", None),
                (1, 0, "a.js", "var Test = ", None),
                (1, 0, "b.js", "{};\n", None),
                (2, 0, "b.js", "Test", None),
                (2, 0, "b.js", ".A", Some("A")),
                (2, 20, "b.js", " = { value: ", Some("A")),
                (-1, -1, "", "1234", None),
                (2, 40, "b.js", " };\n", Some("A")),
                (-1, -1, "", "}());\n", None),
                (-1, -1, "", "/* Generated Source */", None),
            ],
        );
        let input =
            input.to_string_with_source_map(Some(StringPtr::Str(String::from("foo.js"))), None);

        assert_eq!(
            input.source,
            [
                "(function() {",
                "  var Test = {};",
                "Test.A = { value: 1234 };",
                "}());",
                "/* Generated Source */"
            ].join("\n")
        );

        let mut correct_map =
            SourceMapGenerator::new(Some(StringPtr::Str(String::from("foo.js"))), None, false);
        add_mappings_by_params(
            &mut correct_map,
            &[
                (1, 0, Some("a.js"), 1, 0, None),
                (2, 2, Some("a.js"), 1, 0, None),
                (2, 13, Some("b.js"), 1, 0, None),
                (3, 0, Some("b.js"), 2, 0, None),
                (3, 4, Some("b.js"), 2, 0, Some("A")),
                (3, 6, Some("b.js"), 2, 20, Some("A")),
                (3, 18, None, -1, -1, None),
                (3, 22, Some("b.js"), 2, 40, Some("A")),
            ],
        );

        let input_map = input.map;
        let correct_map = correct_map.to_source_map();
        assert_eq!(input_map, correct_map);
        assert_eq!(
            input_map.mappings,
            "AAAA;EAAA,WCAA;AACA,IAAAA,EAAoBA,Y,IAAoBA"
        );
    }

    #[test]
    fn multi_line_source_nodes() {
        let mut input = SourceNode::new(None, None, None, None);
        add_sources_by_params(
            &mut input,
            &[
                (
                    1,
                    0,
                    "a.js",
                    "(function() {\nvar nextLine = 1;\nanotherLine();\n",
                    None,
                ),
                (2, 2, "b.js", "Test.call(this, 123);\n", None),
                (2, 2, "b.js", "this['stuff'] = 'v';\n", None),
                (2, 2, "b.js", "anotherLine();\n", None),
                (-1, -1, "", "/*\nGenerated\nSource\n*/\n", None),
                (3, 4, "c.js", "anotherLine();\n", None),
                (-1, -1, "", "/*\nGenerated\nSource\n*/", None),
            ],
        );
        let input =
            input.to_string_with_source_map(Some(StringPtr::Str(String::from("foo.js"))), None);

        assert_eq!(
            input.source,
            [
                "(function() {",
                "var nextLine = 1;",
                "anotherLine();",
                "Test.call(this, 123);",
                "this['stuff'] = 'v';",
                "anotherLine();",
                "/*",
                "Generated",
                "Source",
                "*/",
                "anotherLine();",
                "/*",
                "Generated",
                "Source",
                "*/"
            ].join("\n")
        );

        let mut correct_map =
            SourceMapGenerator::new(Some(StringPtr::Str(String::from("foo.js"))), None, false);
        add_mappings_by_params(
            &mut correct_map,
            &[
                (1, 0, Some("a.js"), 1, 0, None),
                (2, 0, Some("a.js"), 1, 0, None),
                (3, 0, Some("a.js"), 1, 0, None),
                (4, 0, Some("b.js"), 2, 2, None),
                (5, 0, Some("b.js"), 2, 2, None),
                (6, 0, Some("b.js"), 2, 2, None),
                (11, 0, Some("c.js"), 3, 4, None),
            ],
        );

        let input_map = input.map;
        let correct_map = correct_map.to_source_map();
        assert_eq!(input_map, correct_map);
        assert_eq!(input_map.mappings, "AAAA;AAAA;AAAA;ACCE;AAAA;AAAA;;;;;ACCE");
    }

    #[test]
    fn with_empty_string() {
        let node = SourceNode::new(
            Some((1, 0)),
            Some(StringPtr::Str(String::from("empty.js"))),
            None,
            Some(Node::NString(String::from(""))),
        );
        let result = node.to_string_with_source_map(None, None);
        assert_eq!(result.source, "");
    }

    #[test]
    fn with_consecutive_newlines() {
        let mut input = SourceNode::new(None, None, None, None);
        add_sources_by_params(
            &mut input,
            &[
                (-1, -1, "", "/***/\n\n", None),
                (1, 0, "a.js", "'use strict';\n", None),
                (2, 0, "a.js", "a();", None),
            ],
        );
        let input =
            input.to_string_with_source_map(Some(StringPtr::Str(String::from("foo.js"))), None);

        assert_eq!(
            input.source,
            ["/***/", "", "'use strict';", "a();"].join("\n")
        );

        let mut correct_map =
            SourceMapGenerator::new(Some(StringPtr::Str(String::from("foo.js"))), None, false);
        add_mappings_by_params(
            &mut correct_map,
            &[
                (3, 0, Some("a.js"), 1, 0, None),
                (4, 0, Some("a.js"), 2, 0, None),
            ],
        );

        let input_map = input.map;
        let correct_map = correct_map.to_source_map();
        assert_eq!(input_map, correct_map);
        assert_eq!(input_map.mappings, ";;AAAA;AACA");
    }

    #[test]
    fn with_set_source_content() {
        let mut child_node = SourceNode::new(
            Some((1, 1)),
            Some(StringPtr::Str(String::from("a.js"))),
            None,
            Some(Node::NString(String::from("a"))),
        );
        child_node.set_source_content(
            StringPtr::Str(String::from("a.js")),
            StringPtr::Str(String::from("someContent")),
        );

        let mut node = SourceNode::new(None, None, None, None);
        add_sources_by_params(
            &mut node,
            &[
                (-1, -1, "", "(function () {\n", None),
                (-1, -1, "", "  ", None),
            ],
        );
        node.add(Node::NSourceNode(child_node));
        add_sources_by_params(
            &mut node,
            &[
                (-1, -1, "", "  ", None),
                (1, 1, "b.js", "b", None),
                (-1, -1, "", "}());", None),
            ],
        );
        node.set_source_content(
            StringPtr::Str(String::from("b.js")),
            StringPtr::Str(String::from("otherContent")),
        );

        let map = node
            .to_string_with_source_map(Some(StringPtr::Str(String::from("foo.js"))), None)
            .map;

        assert_eq!(map.sources, ["a.js", "b.js"]);
        assert_eq!(map.sources_content, ["someContent", "otherContent"]);
        assert_eq!(map.mappings, ";EAAC,C,ECAA,C");
    }

    fn add_sources_by_params(sn: &mut SourceNode, params: &[(i32, i32, &str, &str, Option<&str>)]) {
        for param in params {
            let line = param.0;
            let column = param.1;
            let source = String::from(param.2);
            let chunk = String::from(param.3);
            let name = param.4.map(|s| StringPtr::Str(String::from(s)));
            if line >= 0 {
                sn.add(Node::NSourceNode(SourceNode::new(
                    Some((line as usize, column as usize)),
                    Some(StringPtr::Str(source)),
                    name,
                    Some(Node::NString(chunk)),
                )));
            } else {
                sn.add(Node::NString(chunk));
            }
        }
    }

    fn add_mappings_by_params(
        smg: &mut SourceMapGenerator,
        params: &[(i32, i32, Option<&str>, i32, i32, Option<&str>)],
    ) {
        for param in params {
            let generated = (param.0 as usize, param.1 as usize);
            let source = param.2.map(|s| Rc::new(String::from(s)));
            let original = if param.3 >= 0 {
                Some((param.3 as usize, param.4 as usize))
            } else {
                None
            };
            let name = param.5.map(|s| Rc::new(String::from(s)));
            smg.add_mapping(Mapping {
                generated,
                source,
                original,
                name,
            });
        }
    }
}
