//! Вставка заготовок обработчиков в код пользователя. Существующий текст не меняется
//! и не переформатируется: новое только дописывается.

use syn::{ImplItem, Item, Type};

use crate::{Error, Event, model::identifier};

#[derive(Debug)]
pub struct Inserted {
    /// Новый текст файла; `None`, если обработчик уже был.
    pub source: Option<String>,
    /// Строка внутри обработчика (с единицы), куда ставить курсор.
    pub line: usize,
}

pub fn ensure_handler(
    source: &str,
    form: &str,
    handler: &str,
    event: Event,
) -> Result<Inserted, Error> {
    identifier(handler)?;
    let bom = if source.starts_with('\u{feff}') { "\u{feff}" } else { "" };
    let body = &source[bom.len()..];
    let file = syn::parse_file(body).map_err(|e| {
        Error(format!(
            "в файле формы синтаксическая ошибка (строка {}): {e}",
            e.span().start().line
        ))
    })?;

    let blocks: Vec<_> = file
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Impl(block) if block.trait_.is_none() => Some(block),
            _ => None,
        })
        .filter(|block| {
            matches!(&*block.self_ty, Type::Path(p)
                if p.path.segments.last().is_some_and(|s| s.ident == form))
        })
        .collect();

    for block in &blocks {
        for item in &block.items {
            if let ImplItem::Fn(function) = item
                && function.sig.ident == handler
            {
                let open = function.block.brace_token.span.open().start().line;
                let close = function.block.brace_token.span.close().start().line;
                let line = if close > open { open + 1 } else { open };
                return Ok(Inserted { source: None, line });
            }
        }
    }

    let eol = if body.contains("\r\n") { "\r\n" } else { "\n" };
    let stub = [
        format!(
            "    fn {handler}(&mut self, ui: &mut Ui, e: &{}) {{",
            event.args()
        ),
        "        let _ = (ui, e);".to_owned(),
        "    }".to_owned(),
    ]
    .join(eol);

    let (head, tail, text) = match blocks.first() {
        Some(block) => {
            // Перед закрывающей скобкой блока `impl`.
            let close = block.brace_token.span.close().start();
            let at = offset(body, close.line, close.column);
            let head = &body[..at];
            let trimmed = head.trim_end();
            let gap = if trimmed.ends_with('{') { eol.to_owned() } else { eol.repeat(2) };
            (trimmed, &body[at..], format!("{gap}{stub}{eol}"))
        }
        None => {
            let trimmed = body.trim_end();
            let text = format!("{eol}{eol}impl {form} {{{eol}{stub}{eol}}}{eol}");
            (trimmed, "", text)
        }
    };
    let inside = text.find("let _").unwrap_or(0);
    let line = head.matches('\n').count() + text[..inside].matches('\n').count() + 1;
    Ok(Inserted {
        source: Some(format!("{bom}{head}{text}{tail}")),
        line,
    })
}

/// Байтовое смещение по строке (с единицы) и столбцу в символах.
fn offset(text: &str, line: usize, column: usize) -> usize {
    let start: usize = text.split_inclusive('\n').take(line - 1).map(str::len).sum();
    let rest = &text[start..];
    start + rest.char_indices().nth(column).map_or(rest.len(), |(i, _)| i)
}
