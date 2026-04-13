use regex::Regex;
use std::sync::LazyLock;

pub struct SecretPattern {
    pub name: &'static str,
    pub regex: &'static LazyLock<Regex>,
}

static RE_AWS_ACCESS_KEY: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"AKIA[0-9A-Z]{16}").unwrap());

static RE_AWS_SECRET_KEY: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[0-9a-zA-Z/+]{40}").unwrap());

static RE_GITHUB_TOKEN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(ghp_|gho_|ghu_|ghs_|ghr_)[a-zA-Z0-9]{36}").unwrap());

static RE_GITHUB_FINE_GRAINED: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"github_pat_[a-zA-Z0-9_]{82}").unwrap());

static RE_STRIPE_LIVE_SECRET: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"sk_live_[a-zA-Z0-9]{24,}").unwrap());

static RE_STRIPE_TEST_SECRET: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"sk_test_[a-zA-Z0-9]{24,}").unwrap());

static RE_STRIPE_PUBLISHABLE_LIVE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"pk_live_[a-zA-Z0-9]{24,}").unwrap());

static RE_STRIPE_WEBHOOK: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"whsec_[a-zA-Z0-9]{32,}").unwrap());

static RE_GOOGLE_API_KEY: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"AIza[0-9A-Za-z\-_]{35}").unwrap());

static RE_RESEND_API_KEY: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"re_[a-zA-Z0-9]{32,}").unwrap());

static RE_TWILIO_SID: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"AC[a-z0-9]{32}").unwrap());

static RE_MAILGUN_KEY: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"key-[a-z0-9]{32}").unwrap());

static RE_SENDGRID_KEY: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"SG\.[a-zA-Z0-9\-_]{22}\.[a-zA-Z0-9\-_]{43}").unwrap());

static RE_CLOUDINARY_URL: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"cloudinary://[0-9]+:[a-zA-Z0-9]+@").unwrap());

static RE_MONGODB_URI: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"mongodb(\+srv)?://[^:]+:[^@]+@").unwrap());

static RE_POSTGRES_URI: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"postgres(ql)?://[^:]+:[^@]+@").unwrap());

static RE_PRIVATE_KEY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"-----BEGIN (RSA |EC |PGP |DSA )?PRIVATE KEY-----").unwrap()
});

static RE_SUPABASE_JWT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"eyJ[a-zA-Z0-9_-]{100,}").unwrap());

pub fn known_secret_patterns() -> Vec<SecretPattern> {
    vec![
        SecretPattern {
            name: "AWS Access Key",
            regex: &RE_AWS_ACCESS_KEY,
        },
        SecretPattern {
            name: "GitHub Token",
            regex: &RE_GITHUB_TOKEN,
        },
        SecretPattern {
            name: "GitHub Fine-grained Token",
            regex: &RE_GITHUB_FINE_GRAINED,
        },
        SecretPattern {
            name: "Stripe Live Secret Key",
            regex: &RE_STRIPE_LIVE_SECRET,
        },
        SecretPattern {
            name: "Stripe Test Secret Key",
            regex: &RE_STRIPE_TEST_SECRET,
        },
        SecretPattern {
            name: "Stripe Publishable Live Key",
            regex: &RE_STRIPE_PUBLISHABLE_LIVE,
        },
        SecretPattern {
            name: "Stripe Webhook Secret",
            regex: &RE_STRIPE_WEBHOOK,
        },
        SecretPattern {
            name: "Google API Key",
            regex: &RE_GOOGLE_API_KEY,
        },
        SecretPattern {
            name: "Resend API Key",
            regex: &RE_RESEND_API_KEY,
        },
        SecretPattern {
            name: "Twilio Account SID",
            regex: &RE_TWILIO_SID,
        },
        SecretPattern {
            name: "Mailgun API Key",
            regex: &RE_MAILGUN_KEY,
        },
        SecretPattern {
            name: "SendGrid API Key",
            regex: &RE_SENDGRID_KEY,
        },
        SecretPattern {
            name: "Cloudinary URL",
            regex: &RE_CLOUDINARY_URL,
        },
        SecretPattern {
            name: "MongoDB URI with credentials",
            regex: &RE_MONGODB_URI,
        },
        SecretPattern {
            name: "PostgreSQL URI with credentials",
            regex: &RE_POSTGRES_URI,
        },
        SecretPattern {
            name: "Private Key",
            regex: &RE_PRIVATE_KEY,
        },
        SecretPattern {
            name: "Supabase/JWT Token",
            regex: &RE_SUPABASE_JWT,
        },
        // AWS Secret Key is very broad — check last and with entropy
        SecretPattern {
            name: "AWS Secret Key",
            regex: &RE_AWS_SECRET_KEY,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stripe_live_key() {
        let patterns = known_secret_patterns();
        let stripe = patterns
            .iter()
            .find(|p| p.name == "Stripe Live Secret Key")
            .unwrap();
        assert!(stripe
            .regex
            .is_match("sk_live_51Abc2Def3Ghi4Jkl5Mno6Pqr"));
    }

    #[test]
    fn test_github_token() {
        let patterns = known_secret_patterns();
        let gh = patterns
            .iter()
            .find(|p| p.name == "GitHub Token")
            .unwrap();
        assert!(gh
            .regex
            .is_match("ghp_ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefgh12"));
    }

    #[test]
    fn test_aws_access_key() {
        let patterns = known_secret_patterns();
        let aws = patterns
            .iter()
            .find(|p| p.name == "AWS Access Key")
            .unwrap();
        assert!(aws.regex.is_match("AKIAIOSFODNN7EXAMPLE"));
    }

    #[test]
    fn test_no_false_positive_placeholder() {
        let patterns = known_secret_patterns();
        let stripe = patterns
            .iter()
            .find(|p| p.name == "Stripe Live Secret Key")
            .unwrap();
        assert!(!stripe.regex.is_match("your_api_key_here"));
    }
}
