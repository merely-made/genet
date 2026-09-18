# cadency

CSS selector parsing and matching over any element tree.

Implement `Element` for your tree and `PseudoClass` for the dynamic states you
track, then `SelectorList::parse` and `matches`. The crate owns no DOM, no
atoms and no state vocabulary.

Covers type, universal, id, class and attribute selectors; the four
combinators; `:not()`, `:is()`, `:where()`; the structural family including
`:nth-child(an+b of S)`; and `:host`, `:host()`, `::slotted()`, `::part()`
across shadow-tree scopes. `:has()`, named namespace prefixes, quirks mode and
other pseudo-elements are not parsed.

Each parsed selector also reports what a cascade index needs: its specificity,
whether it reaches across a tree scope, the key its rightmost compound
requires, and whether it depends on siblings or on child-list structure.

The name is heraldry's: cadency marks tell which child of a house someone is,
by birth order and line of descent.
