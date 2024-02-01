use super::{GrammarRule, Symbol};

#[derive(PartialEq, Eq)]
struct EarleyItem {
    rule: GrammarRule,
    start: usize,
    current: usize,
}

type StateSet = Vec<EarleyItem>;
type EarleyTable = Vec<(Option<String>, StateSet)>;

pub fn earley_recognize(source: impl Iterator<Item = String>, grammar: &[GrammarRule]) -> bool {
    let table = earley_table(source, grammar);
    match table.last() {
        None => false,
        Some((_, items)) => {
            items.iter().any(|item| item.rule.name == grammar[0].name && item.start == 0 && item.next_unparsed() == None)
        }
    }
}

fn earley_table(mut source: impl Iterator<Item = String>, grammar: &[GrammarRule]) -> EarleyTable {
    let mut token = source.next();
    let mut s = EarleyTable::new();
    if token == None || grammar.is_empty() { return s; }
    s.push(
        (
            token,
            grammar.iter()
            .filter(|r| r.name == grammar[0].name)
            .map(|r| EarleyItem::new(r, 0))
            .collect()
        )
    );


    for i in 0.. {
        if i >= s.len() { break; }
        if s[i].0 != None { s.push((source.next(), StateSet::new())); }
        for j in 0.. {
            if j >= s[i].1.len() { break; }
            match s[i].1[j].next_unparsed() {
                Some(Symbol::Terminal(symbol)) => scan(&mut s, symbol, i, j),
                Some(Symbol::Nonterminal(symbol)) => predict(&mut s, symbol, i, grammar),
                None => complete(&mut s, i, j),
            }
        }
    }
    s
}

fn scan(s: &mut [(Option<String>, Vec<EarleyItem>)], symbol: String, i: usize, j: usize) {
    match &s[i].0 {
        None => return,
        Some(token) => if Symbol::Terminal(symbol).matches(&token) {
            let item = s[i].1[j].advanced();
            push_unique(&mut s[i + 1].1, item);
        }
    }
}

fn predict(s: &mut [(Option<String>, Vec<EarleyItem>)], symbol: String, i: usize, grammar: &[GrammarRule]) {
    for rule in grammar.iter().filter(|r| r.name == symbol) {
        push_unique(&mut s[i].1, EarleyItem::new(rule, i));
    }
}

fn complete(s: &mut [(Option<String>, Vec<EarleyItem>)], i: usize, j: usize) {
    let start = s[i].1[j].start;
    let candidates = std::mem::take(&mut s[start].1);
    let name = s[i].1[j].rule.name.clone();
    for item in candidates.iter()
    .filter(|item| item.next_unparsed() == Some(Symbol::Nonterminal(name.clone()))) {
        push_unique(&mut s[i].1, item.advanced());
    }
    let _ = std::mem::replace(&mut s[start].1, candidates);
}

fn push_unique(set: &mut StateSet, item: EarleyItem) {
    if !set.contains(&item) {
        set.push(item);
    }
}

impl EarleyItem {
    pub fn new(rule: &GrammarRule, start: usize) -> EarleyItem {
        EarleyItem {
            rule: rule.clone(), start, current: 0
        }
    }

    pub fn next_unparsed(&self) -> Option<Symbol> {
        self.rule.components.get(self.current).cloned()
    }

    fn advanced(&self) -> EarleyItem {
        EarleyItem { rule: self.rule.clone(), current: self.current + 1, ..*self }
    }
}