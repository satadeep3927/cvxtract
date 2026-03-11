pub mod error;
pub mod resume;
pub mod schema;

use crate::core::{Model, UnstructuredLoader};
use error::ExtractionError;
use schemars::JsonSchema;
use serde::de::DeserializeOwned;

/// Orchestrates document loading and LLM-powered structured extraction.
///
/// Create an `Extractor` with a [`Model`] provider, then call [`Extractor::extract`]
/// (generic) or [`Extractor::extract_resume`] (built-in Resume type).
///
/// # Example
/// ```no_run
/// use cvxtract::{Extractor, Model};
///
/// # #[tokio::main] async fn main() {
/// let mut extractor = Extractor::new(Some(Model::from_local()));
/// let resume = extractor.extract_resume("cv.pdf".into()).await.unwrap();
/// println!("{:#?}", resume);
/// # }
/// ```
pub struct Extractor {
    model: Model,
}

impl Default for Extractor {
    fn default() -> Self {
        Self::new(None)
    }
}

impl Extractor {
    /// Create a new `Extractor`.
    ///
    /// If `model` is `None`, the local on-device provider (`Model::from_local()`) is used.
    pub fn new(model: Option<Model>) -> Self {
        Self {
            model: model.unwrap_or_else(Model::from_local),
        }
    }

    /// Returns a placeholder JSON value that mirrors the shape of `T`.
    ///
    /// Useful for debugging the auto-generated extraction prompt.
    /// Example output: `{"name": "string", "address": {"city": "string"}}`
    pub fn output_shape<T: JsonSchema>(&self) -> serde_json::Value {
        schema::representation_of::<T>()
    }

    /// Load the document at `path`, prompt the model, and deserialise the response as `T`.
    ///
    /// `T` must implement both [`serde::Deserialize`] and [`schemars::JsonSchema`];
    /// the schema is used to auto-generate the JSON shape hint in the prompt.
    ///
    /// # Errors
    /// Returns [`ExtractionError::LoadError`] if the document cannot be read,
    /// [`ExtractionError::ModelError`] if the model returns an empty response, or
    /// [`ExtractionError::ParseError`] if the JSON cannot be deserialised into `T`.
    pub async fn extract<T>(&mut self, path: std::path::PathBuf) -> Result<T, ExtractionError>
    where
        T: DeserializeOwned + JsonSchema,
    {
        let loader = UnstructuredLoader::new();
        let text = loader.extract_text(path)?;
        let prompt = format!(
            "Extract structured data from the CV text below.\n\
             Rules:\n\
             - Output a single JSON object only. No markdown fences, no explanation, no trailing text.\n\
             - Use null for missing fields. Do not invent data.\n\
             - Dates: output {{\"year\": <int>, \"month\": <int or null>, \"day\": <int or null>}}. Include month/day only if clearly stated in the text.\n\
             - year/month/day must be JSON numbers, never strings.\n\
             - If the end date is ongoing (\"Present\", \"Current\", \"Till date\", etc.), set the entire `end` field to JSON null — do NOT output {{\"year\": null, \"month\": null, \"day\": null}}.\n\
             - Arrays must always be arrays even when there is only one item.\n\
             - Stop immediately after the closing `}}` of the JSON object.\n\
             - Match this exact shape: {shape}\n\n\
             CV:\n{text}",
            shape = self.output_shape::<T>(),
            text = text,
        );
        let raw = self.model.generate(&prompt).await;
        log::debug!("Raw model output: {raw}");
        let value = serde_json::from_str::<T>(strip_fences(&raw))?;
        Ok(value)
    }

    /// Extract into the built-in [`resume::Resume`] type using a fine-tuned prompt.
    ///
    /// This is equivalent to [`Extractor::extract::<Resume>`] but uses a hand-crafted
    /// prompt with detailed rules for education vs. certifications, date handling, etc.,
    /// which produces more reliable results than the auto-generated schema prompt.
    ///
    /// # Errors
    /// Same as [`Extractor::extract`].
    pub async fn extract_resume(
        &mut self,
        path: std::path::PathBuf,
    ) -> Result<resume::Resume, ExtractionError> {
        let loader = UnstructuredLoader::new();
        let text = loader.extract_text(path)?;
        let prompt = format!(
            "Extract every piece of information from the CV below into a single JSON object.\n\
             Rules:\n\
             - Output raw JSON only. No markdown fences, no prose, no trailing text. DO NOT **OVERTHINK**.\n\
             - Use null for any field not found. Do not invent or infer data.\n\
             - Dates: {{\"year\": <int>, \"month\": <int|null>, \"day\": <int|null>}}. Numbers only, never strings.\n\
             - Ongoing roles: set the entire `end` to null, never {{\"year\":null,...}}.\n\
             - Arrays must always be arrays, even for a single item.\n\
             - education: ONLY formal academic qualifications — universities, colleges, schools (e.g. degrees, diplomas, high school). Never put professional certifications here.\n\
             - education.institution: the name of the school/university/college only (e.g. \"MIT\", \"C.B.S.E.\"). NOT the degree title.\n\
             - education.degree: the qualification type (e.g. \"Bachelor of Engineering\", \"XII\", \"X\").\n\
             - education.field: the subject/major (e.g. \"Computer Science & Engineering\"). null if not stated.\n\
             - certifications: ALL professional certs, vendor credentials, and course completions (e.g. \"PeopleSoft\", \"AWS\", \"PMP\", \"L1 Certification in PL/SQL\").\n\
             - certifications.issuer: the body that issued the cert (e.g. \"Oracle\", \"iGATE Solutions Ltd.\"). null if unknown.\n\
             - If a year is completely unknown, use null for year (not 0).\n\
             - company: extract the employer name if mentioned anywhere near the role; null only if truly absent.\n\
             - skills: group by category when the CV has labelled sections; null category when ungrouped.\n\
             - highlights: individual bullet-point achievements/responsibilities per role as separate array items.\n\
             - Stop immediately after the closing `}}`.\n\
             Shape:\n\
             {{\n\
               \"name\": \"string\",\n\
               \"email\": \"string|null\", \"phone\": \"string|null\", \"location\": \"string|null\",\n\
               \"linkedin\": \"string|null\", \"github\": \"string|null\", \"website\": \"string|null\",\n\
               \"summary\": \"string|null\",\n\
               \"experience\": [{{\"company\": \"string|null\", \"role\": \"string\", \"location\": \"string|null\",\n\
                 \"duration\": {{\"start\": {{\"year\":0,\"month\":null,\"day\":null}}, \"end\": {{\"year\":0,\"month\":null,\"day\":null}}|null}},\n\
                 \"summary\": \"string|null\", \"highlights\": [\"string\"]}}],\n\
               \"education\": [{{\"institution\": \"string\", \"degree\": \"string|null\", \"field\": \"string|null\",\n\
                 \"duration\": {{\"start\": {{\"year\":0,\"month\":null,\"day\":null}}, \"end\": null}},\n\
                 \"grade\": \"string|null\"}}],\n\
               \"skills\": [{{\"category\": \"string|null\", \"items\": [\"string\"]}}],\n\
               \"projects\": [{{\"name\": \"string\", \"description\": \"string|null\",\n\
                 \"technologies\": [\"string\"], \"url\": \"string|null\", \"duration\": null}}],\n\
               \"certifications\": [{{\"name\": \"string\", \"issuer\": \"string|null\",\n\
                 \"issued\": null, \"expiry\": null, \"credential_id\": \"string|null\", \"url\": \"string|null\"}}],\n\
               \"languages\": [{{\"language\": \"string\", \"proficiency\": \"string|null\"}}],\n\
               \"awards\": [{{\"title\": \"string\", \"issuer\": \"string|null\", \"date\": null, \"description\": \"string|null\"}}]\n\
             }}\n\n\
             CV:\n{text}",
        );
        let raw = self.model.generate(&prompt).await;
        log::debug!("Raw model output: {raw}");
        let value = serde_json::from_str::<resume::Resume>(strip_fences(&raw))?;
        Ok(value)
    }
}

/// Strip optional markdown code fences from an LLM response.
fn strip_fences(raw: &str) -> &str {
    let s = raw.trim();
    let s = s.strip_prefix("```json").unwrap_or(s);
    let s = s.strip_prefix("```").unwrap_or(s);
    let s = s.trim_end_matches("```");
    s.trim()
}
