use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Year-always-present, month/day optional.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct PartialDate {
    /// null only when the year is genuinely unknown (e.g. ongoing education without dates)
    pub year: Option<u16>,
    pub month: Option<u8>,
    pub day: Option<u8>,
}

/// Half-open date range. `start` and `end` are both null when dates are unknown.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct DateRange {
    /// null only when the start date is genuinely unknown
    pub start: Option<PartialDate>,
    /// null means "Present" / "Current" / ongoing
    pub end: Option<PartialDate>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Experience {
    pub company: Option<String>,
    pub role: Option<String>,
    pub location: Option<String>,
    pub duration: Option<DateRange>,
    /// Short paragraph summarising responsibilities
    pub summary: Option<String>,
    /// Key bullet-point achievements or responsibilities
    pub highlights: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Education {
    pub institution: Option<String>,
    /// e.g. "Bachelor of Science", "MBA"
    pub degree: Option<String>,
    /// e.g. "Computer Science"
    pub field: Option<String>,
    pub duration: Option<DateRange>,
    /// GPA, percentage, or grade classification
    pub grade: Option<String>,
}

/// Skills can be flat or grouped by category.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SkillGroup {
    /// e.g. "Languages", "Frameworks", "Tools" — null if ungrouped
    pub category: Option<String>,
    pub items: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Project {
    pub name: Option<String>,
    pub description: Option<String>,
    pub technologies: Vec<String>,
    pub url: Option<String>,
    pub duration: Option<DateRange>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Certification {
    pub name: Option<String>,
    pub issuer: Option<String>,
    pub issued: Option<PartialDate>,
    pub expiry: Option<PartialDate>,
    pub credential_id: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Language {
    pub language: Option<String>,
    /// e.g. "Native", "Fluent", "Intermediate", "Basic"
    pub proficiency: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Award {
    pub title: Option<String>,
    pub issuer: Option<String>,
    pub date: Option<PartialDate>,
    pub description: Option<String>,
}

/// Top-level resume — covers the vast majority of real-world CVs.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Resume {
    pub name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    /// City, country, or full address
    pub location: Option<String>,
    pub linkedin: Option<String>,
    pub github: Option<String>,
    pub website: Option<String>,
    /// Professional summary or objective statement
    pub summary: Option<String>,
    pub experience: Vec<Experience>,
    pub education: Vec<Education>,
    pub skills: Vec<SkillGroup>,
    pub projects: Vec<Project>,
    pub certifications: Vec<Certification>,
    pub languages: Vec<Language>,
    pub awards: Vec<Award>,
}
