use rome_analyze::context::RuleContext;
use rome_analyze::{declare_rule, Ast, Rule, RuleDiagnostic};
use rome_console::markup;
use rome_js_syntax::jsx_ext::AnyJsxElement;
use rome_rowan::AstNode;

declare_rule! {
    /// Enforce `img` alt prop does not contain the word "image", "picture", or "photo".
    ///
    /// The rule will first check if `aria-hidden` is truthy to determine whether to enforce the rule. If the image is
    /// hidden, then the rule will always succeed.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```jsx,expect_diagnostic
    /// <img src="src" alt="photo content" />;
    /// ```
    ///
    /// ```jsx,expect_diagnostic
    /// <img alt={`picture`} {...this.props} />;
    /// ```
    ///
    /// ```jsx,expect_diagnostic
    /// <img alt="picture of cool person" aria-hidden={false} />;
    /// ```
    ///
    /// ### Valid
    ///
    /// ```jsx
    /// <>
    ///   <img src="src" alt="alt" />
    ///   <img src="src" alt={photo} />
    ///   <img src="bar" aria-hidden alt="Picture of me taking a photo of an image" />
    /// </>
    /// ```
    ///
    pub(crate) NoRedundantAlt {
        version: "12.0.0",
        name: "noRedundantAlt",
        recommended: true,
    }
}

const REDUNDANT_WORDS: [&str; 3] = ["image", "photo", "picture"];

impl Rule for NoRedundantAlt {
    type Query = Ast<AnyJsxElement>;
    type State = ();
    type Signals = Option<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let node = ctx.query();
        if node.name_value_token()?.text_trimmed() != "img" {
            return None;
        }

        if node.has_truthy_attribute("aria-hidden") {
            return None;
        }

        let alt_attribute = node.find_attribute_by_name("alt")?;
        let static_value = alt_attribute.as_static_value()?;
        let alt_string = static_value.as_string_constant()?;
        if REDUNDANT_WORDS.into_iter().any(|word| {
            alt_string
                .split_whitespace()
                .any(|x| x.to_lowercase() == word)
        }) {
            return Some(());
        }

        None
    }

    fn diagnostic(ctx: &RuleContext<Self>, _: &Self::State) -> Option<RuleDiagnostic> {
        let alt_attribute_node = ctx.query().find_attribute_by_name("alt")?;
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                alt_attribute_node.range(),
                markup! {
                    "Avoid the words \"image\", \"picture\", or \"photo\" in " <Emphasis>"img"</Emphasis>" element alt text."
                },
            )
            .note(markup! {
                "Screen readers announce img elements as \"images\", so it is not necessary to redeclare this in alternative text."
            }),
        )
    }
}
