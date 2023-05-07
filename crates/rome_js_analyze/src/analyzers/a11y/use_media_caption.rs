use rome_analyze::context::RuleContext;
use rome_analyze::{declare_rule, Ast, Rule, RuleDiagnostic};
use rome_console::markup;
use rome_js_syntax::jsx_ext::AnyJsxElement;
use rome_js_syntax::{AnyJsxChild, JsxElement, TextRange};
use rome_rowan::AstNode;

declare_rule! {
    /// Enforces that `audio` and `video` elements must have a `track` for captions.
    ///
    /// **ESLint Equivalent:** [media-has-caption](https://github.com/jsx-eslint/eslint-plugin-jsx-a11y/blob/main/docs/rules/media-has-caption.md)
    ///
    /// ## Examples
    ///
    /// ### Invalid
    /// ```jsx,expect_diagnostic
    /// <video />
    /// ```
    ///
    /// ```jsx,expect_diagnostic
    /// <audio>child</audio>
    /// ```
    ///
    /// ### Valid
    ///
    /// ```jsx
    /// <audio>
    ///   <track kind="captions" {...props} />
    /// </audio>
    /// ```
    ///
    /// ```jsx
    /// <video muted {...props}></video>
    /// ```
    pub(crate) UseMediaCaption {
        version: "12.0.0",
        name: "useMediaCaption",
        recommended: true,
    }
}

impl Rule for UseMediaCaption {
    type Query = Ast<AnyJsxElement>;
    type State = TextRange;
    type Signals = Option<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let node = ctx.query();

        if matches!(node.name_value_token()?.text_trimmed(), "video" | "audio") {
            if node.has_truthy_attribute("muted") || node.has_spread_prop() {
                return None;
            }

            match node {
                AnyJsxElement::JsxOpeningElement(_) => {
                    let jsx_element = node.parent::<JsxElement>()?;
                    let has_track = jsx_element
                        .children()
                        .into_iter()
                        .filter_map(|child| match child {
                            AnyJsxChild::JsxElement(element) => {
                                Some(AnyJsxElement::from(element.opening_element().ok()?))
                            }
                            AnyJsxChild::JsxSelfClosingElement(element) => {
                                Some(AnyJsxElement::from(element))
                            }
                            _ => None,
                        })
                        .any(|element| has_valid_track_element(&element).unwrap_or(false));

                    if !has_track {
                        return Some(jsx_element.range());
                    }
                }
                _ => return Some(node.range()),
            }
        }

        None
    }

    fn diagnostic(_: &RuleContext<Self>, range: &Self::State) -> Option<RuleDiagnostic> {
        let diagnostic = RuleDiagnostic::new(
            rule_category!(),
            range,
            markup! {"Provide a "<Emphasis>"track"</Emphasis>" for captions when using "<Emphasis>"audio"</Emphasis>" or "<Emphasis>"video"</Emphasis>" elements."}.to_owned(),
        )
        .note("Captions support users with hearing-impairments. They should be a transcription or translation of the dialogue, sound effects, musical cues, and other relevant audio information.");

        Some(diagnostic)
    }
}

fn has_valid_track_element(element: &AnyJsxElement) -> Option<bool> {
    let is_track_element = element.name_value_token()?.text_trimmed() == "track";
    if let Some(kind_attribute) = element.find_attribute_by_name("kind") {
        let static_value = kind_attribute.as_static_value()?;

        let has_valid_kind = static_value.as_string_constant()?.to_lowercase() == "captions"
            || element.has_trailing_spread_prop(kind_attribute);
        Some(is_track_element && has_valid_kind)
    } else {
        Some(element.has_spread_prop())
    }
}
