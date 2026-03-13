"""
Type stubs for the cvxtract Python extension.

All functions return typed dicts that mirror the Rust Resume structs exactly.
Use these for IDE autocompletion and static type checking (pyright / mypy).

Example::

    import cvxtract

    r = cvxtract.extract_resume("cv.pdf")
    print(r["name"])
    print(r["experience"][0]["company"])
    print(r["experience"][0]["duration"]["start"]["year"])
"""

from typing import TypedDict

# ── Date types ───────────────────────────────────────────────────────────────

class PartialDate(TypedDict):
    """A date where only year is required; month and day are optional."""
    year: int | None
    month: int | None
    day: int | None

class DateRange(TypedDict):
    """A half-open date range. ``end`` being ``None`` means 'Present'."""
    start: PartialDate | None
    end: PartialDate | None

# ── Resume section types ─────────────────────────────────────────────────────

class Experience(TypedDict):
    company: str | None
    role: str | None
    location: str | None
    duration: DateRange | None
    summary: str | None
    highlights: list[str]

class Education(TypedDict):
    institution: str | None
    degree: str | None
    field: str | None
    duration: DateRange | None
    grade: str | None

class SkillGroup(TypedDict):
    """Skills grouped by category. ``category`` is ``None`` when ungrouped."""
    category: str | None
    items: list[str]

class Project(TypedDict):
    name: str | None
    description: str | None
    technologies: list[str]
    url: str | None
    duration: DateRange | None

class Certification(TypedDict):
    name: str | None
    issuer: str | None
    issued: PartialDate | None
    expiry: PartialDate | None
    credential_id: str | None
    url: str | None

class Language(TypedDict):
    language: str | None
    proficiency: str | None

class Award(TypedDict):
    title: str | None
    issuer: str | None
    date: PartialDate | None
    description: str | None

# ── Top-level Resume ──────────────────────────────────────────────────────────

class Resume(TypedDict):
    name: str | None
    email: str | None
    phone: str | None
    location: str | None
    linkedin: str | None
    github: str | None
    website: str | None
    summary: str | None
    experience: list[Experience]
    education: list[Education]
    skills: list[SkillGroup]
    projects: list[Project]
    certifications: list[Certification]
    languages: list[Language]
    awards: list[Award]

# ── Public functions ──────────────────────────────────────────────────────────

def extract_resume(path: str) -> Resume:
    """
    Extract structured resume data from a CV file.

    Loads the file at ``path`` (PDF, DOCX, HTML, or plain text), sends the
    content to GitHub Copilot, and returns the result as a typed Python dict.

    Requires the ``COPILOT_TOKEN`` environment variable to be set.

    :param path: Absolute or relative path to the CV file.
    :returns: A :class:`Resume` dict with all extracted fields.
    :raises RuntimeError: If the file cannot be read, the model call fails,
        or the response cannot be parsed.

    Example::

        r = cvxtract.extract_resume("alice_cv.pdf")
        print(r["name"])                               # "Alice Smith"
        print(r["experience"][0]["role"])              # "Senior Engineer"
        print(r["experience"][0]["duration"]["start"]) # {"year": 2020, ...}
    """
    ...

def extract_resume_json(path: str) -> str:
    """
    Extract structured resume data and return it as a JSON string.

    Equivalent to ``json.dumps(extract_resume(path))`` but avoids the
    Python round-trip — useful when persisting or forwarding the raw JSON.

    :param path: Absolute or relative path to the CV file.
    :returns: Pretty-printed JSON string.
    :raises RuntimeError: Same as :func:`extract_resume`.
    """
    ...
