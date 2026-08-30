//! The BCP 47 language registries, as a generated lookup table.
//!
//! `mediaframe::lang` names languages, scripts and regions, and NONE of that vocabulary is written
//! by hand. Two authority files are vendored verbatim and this module turns them into
//! `mediaframe/src/lang/registry/table.rs`, which is checked in and reviewed like any other source
//! file — the same bargain the pixel-format, colour and codec tables next door already take.
//!
//! ```text
//!   cargo xtask sync      re-fetch both vendored files from their authorities (needs `curl`)
//!   cargo xtask gen-lang  regenerate the table from what is vendored
//!   cargo xtask check     regenerate in memory and DIFF against the checked-in file
//! ```
//!
//! # Two files, and the second one exists because the first cannot answer the question
//!
//! The IANA **language-subtag-registry** is the BCP 47 authority and feeds every table below: the
//! language, script and region rosters, their names, their deprecations and preferred values, the
//! `Suppress-Script` column and the grandfathered tags.
//!
//! It cannot fold `ger` or `deu` onto `de`, because it does not contain either word. BCP 47's own
//! initial-population rule takes a language's ISO 639-1 code where one exists and the 639-3 code
//! only where none does, so the whole ISO 639-2 alpha-3 space for a language that has a two-letter
//! code is ABSENT from the registry — and that is exactly the space a container tags with. An mkv
//! writes the bibliographic `ger`, an mp4 the terminological `deu`, and a registry lookup answers
//! *not registered* to both.
//!
//! So the ISO 639-2 Registration Authority's own table is vendored beside it, and it feeds ONE
//! column: `ALPHA3`, every 639-2 code — bibliographic and terminological — that has a shorter
//! BCP 47 spelling, mapped onto that spelling. Twenty of its rows are the B/T pairs, and all twenty
//! of those languages have two-letter codes, which is why no B/T pair reaches the IANA file at all.
//!
//! # Provenance is CHECKED, not commented
//!
//! [`IANA`] and [`LOC`] carry each file's URL, the date it was fetched and its exact byte count, and
//! `check` verifies the counts against what is on disk. A vendored file swapped for a different one
//! is refused rather than silently regenerating a different table.

use std::{
  collections::{BTreeMap, BTreeSet},
  fmt::Write as _,
  fs,
  path::Path,
  process::Command,
};

/// One vendored authority file: where it came from, when, and how big it was.
///
/// The byte count is the part that does work. A URL in a comment says where a file was MEANT to
/// come from; a count that `check` compares against the bytes on disk says whether the file that is
/// there is still that one.
struct Vendored {
  /// The URL `sync` fetches from.
  url: &'static str,
  /// The path under the workspace root.
  path: &'static str,
  /// The date the checked-in copy was fetched, `YYYY-MM-DD`.
  fetched: &'static str,
  /// The exact size of the checked-in copy. Verified by `check`.
  bytes: usize,
}

/// The BCP 47 registry — the authority for language, script and region subtags, and for every
/// column but `ALPHA3`.
const IANA: Vendored = Vendored {
  url: "https://www.iana.org/assignments/language-subtag-registry/language-subtag-registry",
  path: "xtask/vendor/language-subtag-registry.txt",
  fetched: "2026-08-20",
  bytes: 731_799,
};

/// The ISO 639-2 registrar's own table — the authority for the alpha-3 spellings BCP 47 leaves out.
///
/// Pipe-separated, five columns: `<alpha3-bibliographic>|<alpha3-terminological>|<alpha2>|
/// <English name>|<French name>`, with the terminological column empty where it equals the
/// bibliographic one. Only the first three columns are read — the names are IANA's to give, and
/// taking them from here would be a second table of words for the same subtags.
const LOC: Vendored = Vendored {
  url: "https://www.loc.gov/standards/iso639-2/ISO-639-2_utf-8.txt",
  path: "xtask/vendor/iso639-2.txt",
  fetched: "2026-08-20",
  bytes: 16_010,
};

/// The `Description` the registry gives every subtag and range it reserves for private use.
///
/// A field VALUE of the vendored file rather than knowledge about languages, and the one string
/// this generator matches on. It is what separates `ZZ` — a region record of its own — from an
/// ordinary country code, there being no other column that says so.
const PRIVATE_USE: &str = "Private use";

/// The generated file, relative to the workspace root.
pub(crate) const TABLE_RS: &str = "mediaframe/src/lang/registry/table.rs";

// -------------------------------------------------------------------------------------------
// The three entry points
// -------------------------------------------------------------------------------------------

/// Regenerate the table and write it. Returns the byte count written.
pub(crate) fn generate(root: &Path) -> Result<usize, String> {
  let rendered = render(root)?;
  let path = root.join(TABLE_RS);

  if let Some(parent) = path.parent() {
    fs::create_dir_all(parent)
      .map_err(|e| format!("error: cannot create {}: {e}", parent.display()))?;
  }

  fs::write(&path, &rendered)
    .map_err(|e| format!("error: cannot write {}: {e}", path.display()))?;

  Ok(rendered.len())
}

/// Regenerate the table in memory and report whether the checked-in file is byte-identical.
///
/// The FRESHNESS GATE, and it catches three separate drifts with one comparison: a vendored file
/// replaced without regenerating, a generator changed without regenerating, and a generated file
/// edited by hand.
pub(crate) fn check(root: &Path) -> bool {
  let rendered = match render(root) {
    Ok(text) => text,
    Err(refused) => {
      eprintln!("{refused}");
      return false;
    }
  };

  let path = root.join(TABLE_RS);
  let on_disk = match fs::read_to_string(&path) {
    Ok(text) => text,
    Err(e) => {
      eprintln!("error: cannot read {}: {e}", path.display());
      eprintln!("hint:  run `cargo xtask gen-lang` to write it");
      return false;
    }
  };

  println!("BCP 47 registry: {} ({} bytes)", IANA.fetched, IANA.bytes);
  println!("ISO 639-2 table: {} ({} bytes)", LOC.fetched, LOC.bytes);

  if on_disk == rendered {
    println!("OK: {TABLE_RS} is what the vendored registries say it is.");
    return true;
  }

  eprintln!(
    "FAIL: {TABLE_RS} is stale ({} bytes on disk, {} bytes generated).",
    on_disk.len(),
    rendered.len()
  );
  eprintln!("{}", first_difference(&on_disk, &rendered));
  eprintln!("\nAction: run `cargo xtask gen-lang`.");
  false
}

/// Re-fetch both vendored files from their authorities.
///
/// It does NOT update [`IANA`]/[`LOC`]'s `fetched` and `bytes`, deliberately: those are what `check`
/// verifies against, so bumping them is the act that records a deliberate refresh — and the failing
/// check right after a `sync` is the reminder to do it.
pub(crate) fn sync(root: &Path) -> Result<(), String> {
  for vendored in [IANA, LOC] {
    println!("Fetching {}", vendored.url);
    let fetched = curl(vendored.url)?;
    let path = root.join(vendored.path);

    if let Some(parent) = path.parent() {
      fs::create_dir_all(parent)
        .map_err(|e| format!("error: cannot create {}: {e}", parent.display()))?;
    }
    fs::write(&path, &fetched)
      .map_err(|e| format!("error: cannot write {}: {e}", path.display()))?;

    println!(
      "Wrote {} — {} bytes (the constant says {})",
      vendored.path,
      fetched.len(),
      vendored.bytes
    );
  }

  println!("Now bump `fetched`/`bytes` in xtask/src/lang.rs, then run `cargo xtask gen-lang`.");

  Ok(())
}

/// One URL's bytes, through `curl`.
fn curl(url: &str) -> Result<Vec<u8>, String> {
  let out = Command::new("curl")
    .args(["-sSL", "--fail", url])
    .output()
    .map_err(|e| {
      format!("error: failed to run `curl`: {e}\nhint:  install curl, or fetch the file manually")
    })?;

  if !out.status.success() {
    return Err(format!(
      "error: curl {url} exited with status {}: {}",
      out.status,
      String::from_utf8_lossy(&out.stderr).trim()
    ));
  }

  Ok(out.stdout)
}

/// The first line the two texts disagree on, so a stale table names WHERE it is stale rather than
/// only that it is.
fn first_difference(on_disk: &str, rendered: &str) -> String {
  for (line, (was, is)) in on_disk.lines().zip(rendered.lines()).enumerate() {
    if was != is {
      return format!("first difference at line {}:\n  -{was}\n  +{is}", line + 1);
    }
  }

  String::from("one text is a prefix of the other")
}

// -------------------------------------------------------------------------------------------
// Reading the vendored files
// -------------------------------------------------------------------------------------------

/// One vendored file's text, with its byte count verified against the constant that describes it.
fn vendored(root: &Path, file: &Vendored) -> Result<String, String> {
  let path = root.join(file.path);
  let text =
    fs::read_to_string(&path).map_err(|e| format!("error: cannot read {}: {e}", path.display()))?;

  if text.len() != file.bytes {
    return Err(format!(
      "error: {} is {} bytes and the constant says {} — either re-vendor it with \
       `cargo xtask sync` or bump `bytes` to record a deliberate change",
      file.path,
      text.len(),
      file.bytes
    ));
  }

  Ok(text)
}

/// One record of the IANA registry: a field name to its values, in file order.
///
/// A field may repeat — `Description` does, on every subtag with more than one name — so the values
/// are a list. CONTINUATION LINES are folded here: the format wraps a long field by starting the
/// next line with whitespace, and a reader that took those lines as fields of their own would lose
/// the tail of every long description.
type Record<'a> = BTreeMap<&'a str, Vec<String>>;

/// Every record of the registry, the leading `File-Date` header separated out.
///
/// The format is a sequence of records divided by a line holding exactly `%%`, with the header
/// before the first divider.
fn records(text: &str) -> Result<(String, Vec<Record<'_>>), String> {
  let mut blocks = text.split("%%\n");

  let header = blocks
    .next()
    .ok_or("error: the registry is empty")?
    .strip_prefix("File-Date: ")
    .ok_or("error: the registry does not open with a `File-Date:` header")?
    .trim()
    .to_owned();

  let mut parsed = Vec::new();

  for block in blocks {
    let mut record = Record::new();
    let mut field: Option<&str> = None;

    for line in block.lines() {
      if line.trim().is_empty() {
        continue;
      }

      if line.starts_with([' ', '\t']) {
        let name = field.ok_or("error: a continuation line opens a record")?;
        let held = record
          .get_mut(name)
          .and_then(|values| values.last_mut())
          .ok_or("error: a continuation line follows no value")?;
        held.push(' ');
        held.push_str(line.trim());
        continue;
      }

      let (name, value) = line
        .split_once(':')
        .ok_or_else(|| format!("error: `{line}` is not a `Field: value` line"))?;
      field = Some(name);
      record
        .entry(name)
        .or_default()
        .push(value.trim().to_owned());
    }

    if !record.is_empty() {
      parsed.push(record);
    }
  }

  Ok((header, parsed))
}

/// One field's single value, or a refusal naming the record it was missing from.
fn one<'a>(record: &'a Record<'_>, field: &str) -> Result<&'a str, String> {
  match record.get(field).map(Vec::as_slice) {
    Some([value]) => Ok(value),
    Some(values) => Err(format!("error: `{field}` appears {} times", values.len())),
    None => Err(format!("error: a record has no `{field}`")),
  }
}

/// One field's FIRST value, where the field is allowed to repeat.
fn first<'a>(record: &'a Record<'_>, field: &str) -> Option<&'a str> {
  record.get(field)?.first().map(String::as_str)
}

/// A range record's two bounds, or [`None`] where the subtag names a single value.
///
/// The registry spells a reserved range as `qaa..qtz` in the `Subtag` field itself, which is the
/// one place a subtag is not a subtag.
fn range(subtag: &str) -> Option<(&str, &str)> {
  subtag.split_once("..")
}

// -------------------------------------------------------------------------------------------
// The tables
// -------------------------------------------------------------------------------------------

/// Everything the generated file holds, read off the two vendored files.
///
/// Every map is a [`BTreeMap`] and every set a [`BTreeSet`], which is what makes the emitted arrays
/// sorted — and SORTED is load-bearing twice over: the lookups binary-search them, and a stable
/// order is what lets `check` compare two generations byte for byte.
struct Tables {
  /// The registry's own `File-Date`.
  file_date: String,
  /// Subtag to its first `Description`.
  languages: BTreeMap<String, String>,
  /// Subtag to its `Preferred-Value`.
  language_preferred: BTreeMap<String, String>,
  /// Subtag to its `Suppress-Script`.
  language_suppress_script: BTreeMap<String, String>,
  /// Every subtag carrying a `Deprecated` date.
  language_deprecated: BTreeSet<String>,
  /// The reserved-for-private-use range's two bounds.
  language_private_use: (String, String),
  /// Every ISO 639-2 code with a shorter BCP 47 spelling, mapped onto it.
  alpha3: BTreeMap<String, String>,
  /// Script subtag to its first `Description`.
  scripts: BTreeMap<String, String>,
  /// The script range's two bounds.
  script_private_use: (String, String),
  /// Region subtag to its first `Description`.
  regions: BTreeMap<String, String>,
  /// Region subtag to its `Preferred-Value`.
  region_preferred: BTreeMap<String, String>,
  /// Every region subtag carrying a `Deprecated` date.
  region_deprecated: BTreeSet<String>,
  /// The region ranges' bounds — there are two of them.
  region_private_use: Vec<(String, String)>,
  /// Every region subtag registered INDIVIDUALLY as private use, rather than through a range.
  region_private_use_subtags: BTreeSet<String>,
  /// Grandfathered tag, lowercased, to its `Preferred-Value`.
  grandfathered: BTreeMap<String, String>,
  /// Every grandfathered tag, lowercased, that has NO `Preferred-Value`.
  grandfathered_kept: BTreeSet<String>,
  /// How many times the whole-tag fold can fire before canonicalisation is a fixed point —
  /// PROVEN over the tables above rather than read off a column. See [`grandfathered_hops`].
  grandfathered_hops: usize,
}

/// Read both vendored files into the tables the generated file publishes.
fn tables(root: &Path) -> Result<Tables, String> {
  let registry = vendored(root, &IANA)?;
  let (file_date, records) = records(&registry)?;

  let mut tables = Tables {
    file_date,
    languages: BTreeMap::new(),
    language_preferred: BTreeMap::new(),
    language_suppress_script: BTreeMap::new(),
    language_deprecated: BTreeSet::new(),
    language_private_use: (String::new(), String::new()),
    alpha3: BTreeMap::new(),
    scripts: BTreeMap::new(),
    script_private_use: (String::new(), String::new()),
    regions: BTreeMap::new(),
    region_preferred: BTreeMap::new(),
    region_deprecated: BTreeSet::new(),
    region_private_use: Vec::new(),
    region_private_use_subtags: BTreeSet::new(),
    grandfathered: BTreeMap::new(),
    grandfathered_kept: BTreeSet::new(),
    grandfathered_hops: 0,
  };

  for record in &records {
    match one(record, "Type")? {
      "language" => language(record, &mut tables)?,
      "script" => script(record, &mut tables)?,
      "region" => region(record, &mut tables)?,
      "grandfathered" => grandfathered(record, &mut tables)?,
      // `extlang`, `variant` and `redundant` are read by nothing. The first two have no seat in
      // `LanguageId` — an extlang and a variant both ride the lossless `rest` tail — and the third
      // is a list of tags that were once registered whole and are now ordinary compositions, which
      // is a fact about the registry's history rather than about a tag being read today.
      _ => {}
    }
  }

  alpha3(&vendored(root, &LOC)?, &mut tables)?;
  audit(&tables)?;
  tables.grandfathered_hops = grandfathered_hops(&tables)?;

  Ok(tables)
}

/// One `language` record.
fn language(record: &Record<'_>, tables: &mut Tables) -> Result<(), String> {
  let subtag = one(record, "Subtag")?;

  if let Some((low, high)) = range(subtag) {
    tables.language_private_use = (low.to_owned(), high.to_owned());
    return Ok(());
  }

  let name =
    first(record, "Description").ok_or_else(|| format!("error: `{subtag}` has no description"))?;
  tables.languages.insert(subtag.to_owned(), name.to_owned());

  if let Some(preferred) = record.get("Preferred-Value") {
    let preferred = preferred.first().expect("a present field has a value");
    tables
      .language_preferred
      .insert(subtag.to_owned(), preferred.clone());
  }

  if let Some(script) = record.get("Suppress-Script") {
    let script = script.first().expect("a present field has a value");
    tables
      .language_suppress_script
      .insert(subtag.to_owned(), script.clone());
  }

  if record.contains_key("Deprecated") {
    tables.language_deprecated.insert(subtag.to_owned());
  }

  Ok(())
}

/// One `script` record.
fn script(record: &Record<'_>, tables: &mut Tables) -> Result<(), String> {
  let subtag = one(record, "Subtag")?;

  if let Some((low, high)) = range(subtag) {
    tables.script_private_use = (low.to_owned(), high.to_owned());
    return Ok(());
  }

  let name =
    first(record, "Description").ok_or_else(|| format!("error: `{subtag}` has no description"))?;
  tables.scripts.insert(subtag.to_owned(), name.to_owned());

  Ok(())
}

/// One `region` record.
fn region(record: &Record<'_>, tables: &mut Tables) -> Result<(), String> {
  let subtag = one(record, "Subtag")?;

  if let Some((low, high)) = range(subtag) {
    tables
      .region_private_use
      .push((low.to_owned(), high.to_owned()));
    return Ok(());
  }

  let name =
    first(record, "Description").ok_or_else(|| format!("error: `{subtag}` has no description"))?;
  tables.regions.insert(subtag.to_owned(), name.to_owned());

  // A region is the ONE subtag kind the registry reserves for private use both by RANGE and by
  // individual record: `AA` and `ZZ` carry rows of their own with the description below. The word
  // is the registry's own vocabulary rather than knowledge about places, and the audit refuses a
  // file in which a language or a script grows one — so the asymmetry stays checked rather than
  // assumed.
  if PRIVATE_USE.eq_ignore_ascii_case(name) {
    tables.region_private_use_subtags.insert(subtag.to_owned());
  }

  if let Some(preferred) = record.get("Preferred-Value") {
    let preferred = preferred.first().expect("a present field has a value");
    tables
      .region_preferred
      .insert(subtag.to_owned(), preferred.clone());
  }

  if record.contains_key("Deprecated") {
    tables.region_deprecated.insert(subtag.to_owned());
  }

  Ok(())
}

/// One `grandfathered` record.
///
/// The tag is LOWERCASED on the way in, because it is looked up against a whole tag a caller sent
/// and case is not part of a tag's identity — `I-KLINGON` names what `i-klingon` names.
fn grandfathered(record: &Record<'_>, tables: &mut Tables) -> Result<(), String> {
  let tag = one(record, "Tag")?.to_ascii_lowercase();

  match record.get("Preferred-Value") {
    Some(preferred) => {
      let preferred = preferred.first().expect("a present field has a value");
      tables.grandfathered.insert(tag, preferred.clone());
    }
    None => {
      tables.grandfathered_kept.insert(tag);
    }
  }

  Ok(())
}

/// The ISO 639-2 table's third column, read as a fold onto the shortest BCP 47 spelling.
///
/// Both alpha-3 columns are folded onto the alpha-2 one, which is what makes `ger` and `deu` one
/// entry each rather than a chain: a bibliographic code reaches the two-letter code DIRECTLY, so no
/// lookup here needs a second hop.
///
/// A row with an EMPTY alpha-2 column contributes nothing. Its alpha-3 code is the shortest
/// spelling there is, so IANA carries it and no fold is wanted — `haw` stays `haw`.
///
/// The one range row (`qaa-gtz`, reserved for local use) is skipped by the same rule that skips the
/// registry's own range records: it has no alpha-2 column either.
fn alpha3(text: &str, tables: &mut Tables) -> Result<(), String> {
  for line in text.trim_start_matches('\u{feff}').lines() {
    if line.trim().is_empty() {
      continue;
    }

    let columns: Vec<&str> = line.split('|').collect();
    let [bibliographic, terminological, alpha2, ..] = columns.as_slice() else {
      return Err(format!(
        "error: `{line}` is not a five-column ISO 639-2 row"
      ));
    };

    if alpha2.is_empty() {
      continue;
    }

    for code in [bibliographic, terminological] {
      if code.is_empty() {
        continue;
      }
      tables
        .alpha3
        .insert((*code).to_owned(), (*alpha2).to_owned());
    }
  }

  Ok(())
}

/// The premises the generated table is read under, checked against the data rather than assumed.
///
/// Each one is something a lookup in `mediaframe::lang::registry` DEPENDS on, and each is a
/// property of today's files rather than of the format — so the place to find out that one stopped
/// holding is here, at generation, and not in a wrong answer months later.
fn audit(tables: &Tables) -> Result<(), String> {
  if tables.languages.is_empty() || tables.scripts.is_empty() || tables.regions.is_empty() {
    return Err(String::from("error: a roster came out empty"));
  }

  // ONE HOP. `Language`'s fold applies `Preferred-Value` once, so a preferred value that is itself
  // deprecated would leave a value the fold calls canonical and the registry calls superseded.
  for (subtag, preferred) in &tables.language_preferred {
    if tables.language_preferred.contains_key(preferred) {
      return Err(format!(
        "error: `{subtag}` prefers `{preferred}`, which prefers something else — the fold is one \
         hop"
      ));
    }
    if !tables.languages.contains_key(preferred) {
      return Err(format!(
        "error: `{subtag}` prefers `{preferred}`, which is not registered"
      ));
    }
  }

  for (subtag, preferred) in &tables.region_preferred {
    if tables.region_preferred.contains_key(preferred) {
      return Err(format!(
        "error: `{subtag}` prefers `{preferred}`, which prefers something else — the fold is one \
         hop"
      ));
    }
    if !tables.regions.contains_key(preferred) {
      return Err(format!(
        "error: `{subtag}` prefers `{preferred}`, which is not registered"
      ));
    }
  }

  // The two files AGREE, which is the premise the second one exists under: it supplies spellings
  // IANA leaves out, and it must not supply a different answer for one IANA carries.
  for (code, shortest) in &tables.alpha3 {
    if tables.languages.contains_key(code) {
      return Err(format!(
        "error: ISO 639-2 folds `{code}` onto `{shortest}`, and BCP 47 registers `{code}` itself"
      ));
    }
    if !tables.languages.contains_key(shortest) {
      return Err(format!(
        "error: ISO 639-2 folds `{code}` onto `{shortest}`, which BCP 47 does not register"
      ));
    }
  }

  // Every range is present and well formed, since the private-use predicates compare against these
  // bounds and an empty bound would admit everything.
  let mut ranges = vec![&tables.language_private_use, &tables.script_private_use];
  ranges.extend(&tables.region_private_use);

  for (low, high) in ranges {
    if low.is_empty() || low.len() != high.len() || low > high {
      return Err(format!("error: `{low}..{high}` is not a usable range"));
    }
  }

  // A REGION is the only subtag kind with individually registered private-use rows, and the two
  // predicates read that asymmetry rather than a range test alone. If a language or a script ever
  // grows one, the fold that ignores it would be silently wrong — so the absence is checked here
  // rather than assumed, and the generator is where a registry bump is noticed.
  for (kind, names) in [("language", &tables.languages), ("script", &tables.scripts)] {
    if let Some((subtag, _)) = names
      .iter()
      .find(|(_, name)| PRIVATE_USE.eq_ignore_ascii_case(name))
    {
      return Err(format!(
        "error: the registry now registers the {kind} `{subtag}` as private use on its own, and \
         only a region used to have one — `{kind}_is_private_use` tests a RANGE and would answer \
         `false` to it"
      ));
    }
  }

  if tables.region_private_use_subtags.is_empty() {
    return Err(String::from(
      "error: no region is registered as private use on its own, and `AA` and `ZZ` used to be",
    ));
  }

  // A `Suppress-Script` names a script `LanguageId`'s fold then compares against, so one naming no
  // registered script would be a fold that never fires and never says why.
  for (subtag, script) in &tables.language_suppress_script {
    if !tables.scripts.contains_key(script) {
      return Err(format!(
        "error: `{subtag}` suppresses `{script}`, which is not a registered script"
      ));
    }
  }

  Ok(())
}

// -------------------------------------------------------------------------------------------
// The fold graph, and the bound it proves
// -------------------------------------------------------------------------------------------

/// The four seats a tag composes to — the generator's model of `LanguageId::composed`.
///
/// It exists because the property below is about the tables' INTERACTION, and the interaction only
/// shows once the folds are applied together: a seat fold rewrites a subtag, the suppression
/// deletes one, and the tag those two leave is what the whole-tag fold is then asked about.
struct Composed {
  language: String,
  script: Option<String>,
  region: Option<String>,
  rest: Option<String>,
}

impl Composed {
  /// The canonical text, lower-cased — which is the spelling the whole-tag table is keyed by.
  ///
  /// It carries no WIDTH guard where the crate's own lookup skips a tag too wide for its stack
  /// buffer, and the asymmetry is deliberately in the safe direction: a text this finds in the
  /// table that the runtime would have skipped can only make the bound LONGER than the loop needs.
  /// The two agree today because no grandfathered tag is that wide, which the crate's
  /// `the_grandfathered_table_fits_the_lookup_buffer` is the pin for.
  fn lowered(&self) -> String {
    let mut text = self.language.clone();

    for part in [
      self.script.as_ref(),
      self.region.as_ref(),
      self.rest.as_ref(),
    ]
    .into_iter()
    .flatten()
    {
      text.push('-');
      text.push_str(part);
    }

    text.to_ascii_lowercase()
  }
}

/// One tag through the composition rules, or [`None`] where the door would refuse it.
///
/// **A SECOND IMPLEMENTATION of `LanguageId::composed`, and deliberately so**: this one reads the
/// tables the generator is ABOUT TO EMIT, so the property it proves is a property of those tables
/// rather than of the ones already checked in. The two are pinned to each other by
/// `the_generated_hop_bound_is_the_chain_this_registry_has`, which recomputes the same walk through
/// the crate's own composition and asserts it reaches the number emitted here — so a drift between
/// the model and the fold fails the test suite rather than hiding.
fn compose(tag: &str, tables: &Tables) -> Option<Composed> {
  let parts: Vec<&str> = tag.split('-').collect();

  if parts.iter().any(|part| part.is_empty()) {
    return None;
  }

  // The language seat: two to eight ASCII letters, lower-cased, then alpha-3 and `Preferred-Value`
  // in sequence — the order `Language::new` applies them in.
  let head = parts[0];
  if !(2..=8).contains(&head.len()) || !head.chars().all(|c| c.is_ascii_alphabetic()) {
    return None;
  }
  let lowered = head.to_ascii_lowercase();
  let shortest = tables.alpha3.get(&lowered).unwrap_or(&lowered);
  let language = tables
    .language_preferred
    .get(shortest)
    .unwrap_or(shortest)
    .clone();

  let mut at = 1;

  let script = match parts.get(at) {
    Some(part) if script_shaped(part) => {
      at += 1;
      Some(titlecase(part))
    }
    _ => None,
  };

  let region = match parts.get(at) {
    Some(part) if region_shaped(part) => {
      at += 1;
      // A digit has no case, so only the letter arm folds — `Region::new`'s own asymmetry.
      let canonical = match part.len() == 2 {
        true => part.to_ascii_uppercase(),
        false => (*part).to_owned(),
      };
      Some(
        tables
          .region_preferred
          .get(&canonical)
          .cloned()
          .unwrap_or(canonical),
      )
    }
    _ => None,
  };

  let consumed: usize = parts[..at].iter().map(|part| part.len() + 1).sum();
  let rest = match consumed < tag.len() {
    true => {
      let tail = &tag[consumed..];
      let admitted = tail.split('-').all(|part| {
        (1..=8).contains(&part.len()) && part.chars().all(|c| c.is_ascii_alphanumeric())
      });
      if !admitted {
        return None;
      }
      Some(tail.to_owned())
    }
    false => None,
  };

  // `Suppress-Script`, under the same guard the fold carries: a tail that opens script-SHAPED would
  // climb into the slot the fold vacates, so the script is retained instead.
  let script = match (&script, tables.language_suppress_script.get(&language)) {
    (Some(declared), Some(implied))
      if declared == implied
        && !rest
          .as_deref()
          .and_then(|tail| tail.split('-').next())
          .is_some_and(script_shaped) =>
    {
      None
    }
    _ => script,
  };

  Some(Composed {
    language,
    script,
    region,
    rest,
  })
}

/// Exactly four ASCII letters — the script position's shape.
fn script_shaped(part: &str) -> bool {
  part.len() == 4 && part.chars().all(|c| c.is_ascii_alphabetic())
}

/// Two ASCII letters or three ASCII digits — the region position's shape.
fn region_shaped(part: &str) -> bool {
  (part.len() == 2 && part.chars().all(|c| c.is_ascii_alphabetic()))
    || (part.len() == 3 && part.chars().all(|c| c.is_ascii_digit()))
}

/// One capital then lower — the case ISO 15924 and the registry both spell a script in.
///
/// Asked only of a [`script_shaped`] subtag, which is four characters, so the first byte it
/// re-cases is always there.
fn titlecase(part: &str) -> String {
  let mut out = part.to_ascii_lowercase();
  out[..1].make_ascii_uppercase();
  out
}

/// **THE TERMINATION PROOF**: how many times the whole-tag fold can fire before canonicalisation
/// is a fixed point, over the tables about to be emitted.
///
/// # Why a proof rather than a rule
///
/// `LanguageId::new` applies a SEQUENCE of folds — the seat folds, `Suppress-Script`, and this
/// table over the whole tag — and any fold whose output lands in an earlier fold's PREIMAGE moves
/// the tag again. `en-Latn-GB-oed` is not grandfathered as written; the suppression drops `Latn`
/// and leaves `en-GB-oed`, which IS, and `eng-GB-oed` reaches the same text through the alpha-3
/// fold. A canonicalisation that applied each fold once would render text that parses back as a
/// DIFFERENT value, and the rendering is what serde and the wire codec store.
///
/// So the fold is applied to a FIXED POINT, and what has to be true for that to terminate is a
/// property of the registry rather than of the code — which is why it is settled here, where the
/// registry is being read, and carried into the crate as a number.
///
/// # The graph, and why walking the table is exhaustive
///
/// The loop's first hop can start from any tag a caller sends, but it always LANDS on a
/// `Preferred-Value` of this table — so every hop after the first is an edge of a graph whose
/// nodes are the table's own rows. Walking every row therefore covers every chain any input can
/// take: this starts at each grandfathered tag, composes its replacement, renders it, and asks the
/// table again, until the rendering names no row.
///
/// # Errors
///
/// A CYCLE, which is a design letter and not a hack site — canonicalisation would have no fixed
/// point to reach and no bound would be honest. And a `Preferred-Value` the composition rules
/// cannot read, which would make the fold's own output a tag the door refuses.
fn grandfathered_hops(tables: &Tables) -> Result<usize, String> {
  let mut deepest = 0usize;

  for start in tables.grandfathered.keys() {
    let mut chain: Vec<&str> = vec![start.as_str()];
    let mut here: &str = start.as_str();

    while let Some(preferred) = tables.grandfathered.get(here) {
      let composed = compose(preferred, tables).ok_or_else(|| {
        format!(
          "error: the grandfathered tag `{here}` prefers `{preferred}`, which the composition \
           rules cannot read — a `Preferred-Value` has to be a tag the whole-tag door admits, or \
           the fold turns a valid tag into a refusal"
        )
      })?;

      let Some((next, _)) = tables.grandfathered.get_key_value(&composed.lowered()) else {
        break;
      };

      if chain.contains(&next.as_str()) {
        chain.push(next.as_str());
        return Err(format!(
          "error: the whole-tag fold CYCLES — {} — so canonicalisation has no fixed point to \
           reach. This is a design letter rather than a hack site: `LanguageId::new` iterates the \
           fold until the rendering is stable, and a cycle means it never is",
          chain.join(" -> ")
        ));
      }

      chain.push(next.as_str());
      here = next.as_str();
    }

    // Every tag in the chain cost one hop: the first is the fold that reached this row, and each
    // later one the fold the previous row's own rendering opened.
    deepest = deepest.max(chain.len());
  }

  Ok(deepest)
}

// -------------------------------------------------------------------------------------------
// Emission
// -------------------------------------------------------------------------------------------

/// The generated file's whole text, formatted the way `cargo fmt` would leave it.
///
/// The generator writes valid Rust and makes no attempt to write FORMATTED Rust — every row is
/// emitted flush left and the formatter is what indents them. That is the same last step
/// `gen-codec` takes and it buys the same thing: what lands on disk is a fixpoint of the formatter
/// `cargo fmt --check` runs, so a generated file never fails the format gate.
fn render(root: &Path) -> Result<String, String> {
  let tables = tables(root)?;
  let mut out = String::with_capacity(1 << 19);

  header(&tables, &mut out);

  pairs(
    &mut out,
    "LANGUAGES",
    "Every registered primary language subtag, with the first `Description` the registry gives it.",
    "Membership here IS registration: `Language::is_registered` is this lookup answering `Some`.",
    &tables.languages,
    Some("MAX"),
  );
  pairs(
    &mut out,
    "LANGUAGE_PREFERRED",
    "Every deprecated language subtag that names a replacement, mapped onto it.",
    "The registry's `Preferred-Value`. A deprecated subtag ABSENT from here has no replacement and \
     folds to itself.",
    &tables.language_preferred,
    Some("MAX"),
  );
  pairs(
    &mut out,
    "LANGUAGE_SUPPRESS_SCRIPT",
    "Every language subtag that implies a script, mapped onto the script it implies.",
    "The registry's `Suppress-Script`. `en` implies `Latn`, so `en-Latn` composes as `en`; `zh` is \
     ABSENT, so `zh-Hans` composes as itself.",
    &tables.language_suppress_script,
    Some("MAX"),
  );
  words(
    &mut out,
    "LANGUAGE_DEPRECATED",
    "Every language subtag the registry has deprecated, whether or not it names a replacement.",
    "Deprecation and replacement are two columns: 120 of these name nothing to fold onto, and stay \
     themselves.",
    &tables.language_deprecated,
  );
  bounds(
    &mut out,
    "LANGUAGE_PRIVATE_USE",
    "The language range the registry reserves for private use, as its inclusive bounds.",
    &tables.language_private_use,
  );
  pairs(
    &mut out,
    "ALPHA3",
    "Every ISO 639-2 code with a shorter BCP 47 spelling, mapped onto that spelling.",
    "The ONE column the second vendored file feeds — see this workspace's `xtask` docs for why BCP \
     47 cannot supply it. Both the bibliographic and the terminological code of a pair are here, \
     each pointing straight at the two-letter code: `ger` and `deu` both answer `de`.",
    &tables.alpha3,
    Some("MAX"),
  );
  pairs(
    &mut out,
    "SCRIPTS",
    "Every registered script subtag, with the first `Description` the registry gives it.",
    "ISO 15924, in the registry's own Titlecase.",
    &tables.scripts,
    Some("WIDTH"),
  );
  bounds(
    &mut out,
    "SCRIPT_PRIVATE_USE",
    "The script range the registry reserves for private use, as its inclusive bounds.",
    &tables.script_private_use,
  );
  pairs(
    &mut out,
    "REGIONS",
    "Every registered region subtag, with the first `Description` the registry gives it.",
    "Two grammars in one table: ISO 3166-1 alpha-2 in upper case, and UN M.49 as three digits.",
    &tables.regions,
    Some("AREA"),
  );
  pairs(
    &mut out,
    "REGION_PREFERRED",
    "Every deprecated region subtag that names a replacement, mapped onto it.",
    "`BU` folds onto `MM`. A deprecated region ABSENT from here has no replacement and stays \
     itself.",
    &tables.region_preferred,
    Some("AREA"),
  );
  words(
    &mut out,
    "REGION_DEPRECATED",
    "Every region subtag the registry has deprecated, whether or not it names a replacement.",
    "`AN`, `CS`, `NT`, `SU` and `YU` are the ones that name none — a country that dissolved into \
     several has no single successor to fold onto.",
    &tables.region_deprecated,
  );
  ranges(
    &mut out,
    "REGION_PRIVATE_USE",
    "The region ranges the registry reserves for private use, as inclusive bounds.",
    "TWO of them, where a language and a script have one each.",
    &tables.region_private_use,
  );
  words(
    &mut out,
    "REGION_PRIVATE_USE_SUBTAGS",
    "Every region subtag the registry reserves for private use INDIVIDUALLY, outside the ranges.",
    "`AA` and `ZZ`, each a record of its own carrying the description `Private use`. A region is \
     the only subtag kind with any, which is why it is the only one whose private-use question is \
     not a range test alone — and the generator refuses a registry in which a language or a script \
     grows one.",
    &tables.region_private_use_subtags,
  );
  pairs(
    &mut out,
    "GRANDFATHERED",
    "Every grandfathered tag that names a replacement, mapped onto it. Lower-cased on both sides \
     of the lookup.",
    "A whole TAG rather than a subtag, which is what makes this table `LanguageId`'s and not \
     `Language`'s.",
    &tables.grandfathered,
    Some("GRANDFATHERED_MAX"),
  );
  words(
    &mut out,
    "GRANDFATHERED_KEPT",
    "Every grandfathered tag that names NO replacement.",
    "Five of them. Two parse as ordinary compositions once the tag is read subtag by subtag \
     (`cel-gaulish`, `zh-min`); the three beginning `i-` do not, a one-letter primary subtag being \
     outside the grammar every other tag is read by.",
    &tables.grandfathered_kept,
  );
  count(
    &mut out,
    "MAX_GRANDFATHERED_HOPS",
    "How many times the whole-tag fold can fire before a canonicalisation is a FIXED POINT.",
    "PROVEN at generation rather than chosen: the generator walks every grandfathered tag through \
     the composition rules — the seat folds, `Suppress-Script` and this table — and follows the \
     chain each fold's own output opens, refusing a registry in which one CYCLES. \
     `LanguageId::new` iterates at most this many times, so a registry bump that lengthened a \
     chain fails `cargo xtask check` rather than production.",
    tables.grandfathered_hops,
  );

  let edition = crate::read_mediaframe_edition(root)?;
  crate::run_rustfmt(&out, &edition)
}

/// The generated file's module documentation — the provenance a reader lands on first.
fn header(tables: &Tables, out: &mut String) {
  let _ = write!(
    out,
    "//! The vendored language registries, as sorted lookup tables.\n\
     //!\n\
     //! **GENERATED by `cargo xtask gen-lang` — do not edit.** Every word below comes from a file \
     vendored\n\
     //! under `xtask/vendor/`, and `cargo xtask check` refuses if this file and those files \
     disagree.\n\
     //!\n\
     //! Every table but [`ALPHA3`] comes from the BCP 47 registry, fetched {iana_fetched} at\n\
     //! {iana_bytes} bytes, whose own `File-Date` is **{file_date}**:\n\
     //!\n\
     //! <{iana}>\n\
     //!\n\
     //! [`ALPHA3`] alone comes from the ISO 639-2 registrar's table, fetched {loc_fetched} at\n\
     //! {loc_bytes} bytes:\n\
     //!\n\
     //! <{loc}>\n\
     //!\n\
     //! Every array is sorted by its first element, which is what [`super`]'s lookups \
     binary-search\n\
     //! and what makes two generations of this file comparable byte for byte.\n\
     \n",
    iana = IANA.url,
    iana_fetched = IANA.fetched,
    iana_bytes = IANA.bytes,
    loc = LOC.url,
    loc_fetched = LOC.fetched,
    loc_bytes = LOC.bytes,
    file_date = tables.file_date,
  );

  let _ = write!(
    out,
    "/// The `File-Date` of the vendored BCP 47 registry these tables were generated from.\n\
     pub const FILE_DATE: &str = {:?};\n\n",
    tables.file_date
  );

  for line in wrapped(
    "The family's own inline ASCII seats — `MAX` (language), `WIDTH` (script), `AREA` (region) \
     and `GRANDFATHERED_MAX` (a whole grandfathered tag) — each already the width its own type is \
     stored at, so a key column that rides one is stored the same way the value it was read from \
     is.",
  ) {
    let _ = writeln!(out, "// {line}");
  }
  let _ = writeln!(
    out,
    "use crate::lang::{{\n  id::GRANDFATHERED_MAX,\n  region::AREA,\n  script::WIDTH,\n  \
     subtag::{{Ascii, MAX}},\n}};\n"
  );
}

/// One `&[(K, &str)]` table, `K` being `&str` or, where `key_seat` names one, the family's own
/// inline `Ascii<N>` seat.
///
/// `key_seat` is the LOCAL NAME (after this file's own `use`, see [`header`]) of the `usize`
/// constant the key column is stored inline at — `MAX` for a language-shaped key, `WIDTH` for a
/// script, `AREA` for a region, `GRANDFATHERED_MAX` for a whole grandfathered tag. This generator
/// never reads that constant's VALUE: `mediaframe`'s own `Ascii::literal` asserts a key fits it at
/// that CRATE's compile time, which is what turns "the registry grew a key past its seat" into a
/// build failure there rather than a silent truncation here.
fn pairs(
  out: &mut String,
  name: &str,
  summary: &str,
  detail: &str,
  rows: &BTreeMap<String, String>,
  key_seat: Option<&str>,
) {
  doc(out, name, summary, detail, rows.len());
  match key_seat {
    Some(seat) => {
      let _ = writeln!(
        out,
        "pub(crate) static {name}: &[(Ascii<{seat}>, &str)] = &["
      );
      for (key, value) in rows {
        let _ = writeln!(out, "(Ascii::literal({key:?}), {value:?}),");
      }
    }
    None => {
      let _ = writeln!(out, "pub(crate) static {name}: &[(&str, &str)] = &[");
      for (key, value) in rows {
        let _ = writeln!(out, "({key:?}, {value:?}),");
      }
    }
  }
  let _ = writeln!(out, "];\n");
}

/// One `&[&str]` table.
fn words(out: &mut String, name: &str, summary: &str, detail: &str, rows: &BTreeSet<String>) {
  doc(out, name, summary, detail, rows.len());
  let _ = writeln!(out, "pub(crate) static {name}: &[&str] = &[");
  for word in rows {
    let _ = writeln!(out, "{word:?},");
  }
  let _ = writeln!(out, "];\n");
}

/// One `usize` constant the generator PROVED rather than read off a column.
///
/// It has no row count to close with, which is the difference from [`doc`]: what a reader wants
/// beside a number the generator computed is what was walked to reach it.
fn count(out: &mut String, name: &str, summary: &str, detail: &str, value: usize) {
  let _ = writeln!(out, "/// {summary}");
  let _ = writeln!(out, "///");
  for line in wrapped(detail) {
    let _ = writeln!(out, "/// {line}");
  }
  let _ = writeln!(out, "pub(crate) const {name}: usize = {value};\n");
}

/// One inclusive range, as a constant pair.
fn bounds(out: &mut String, name: &str, summary: &str, range: &(String, String)) {
  let _ = writeln!(out, "/// {summary}");
  let _ = writeln!(
    out,
    "pub(crate) const {name}: (&str, &str) = ({:?}, {:?});\n",
    range.0, range.1
  );
}

/// A table of inclusive ranges.
fn ranges(out: &mut String, name: &str, summary: &str, detail: &str, rows: &[(String, String)]) {
  doc(out, name, summary, detail, rows.len());
  let _ = writeln!(out, "pub(crate) static {name}: &[(&str, &str)] = &[");
  for (low, high) in rows {
    let _ = writeln!(out, "({low:?}, {high:?}),");
  }
  let _ = writeln!(out, "];\n");
}

/// One table's doc comment, closing with the row count the generation produced.
///
/// The count is emitted rather than described because it is the one number a reader can check the
/// vendored file against, and the one a re-generation moves when the authority grows.
fn doc(out: &mut String, name: &str, summary: &str, detail: &str, rows: usize) {
  let _ = writeln!(out, "/// {summary}");
  let _ = writeln!(out, "///");
  for line in wrapped(detail) {
    let _ = writeln!(out, "/// {line}");
  }
  let _ = writeln!(out, "///");
  let _ = writeln!(out, "/// {rows} rows, sorted. Generated into `{name}`.");
}

/// A sentence broken into lines that fit the formatter's width, so the emitted doc comment is one
/// `rustfmt` leaves alone.
fn wrapped(text: &str) -> Vec<String> {
  let mut lines = Vec::new();
  let mut line = String::new();

  for word in text.split_whitespace() {
    if !line.is_empty() && line.len() + 1 + word.len() > 96 {
      lines.push(std::mem::take(&mut line));
    }
    if !line.is_empty() {
      line.push(' ');
    }
    line.push_str(word);
  }

  if !line.is_empty() {
    lines.push(line);
  }

  lines
}
