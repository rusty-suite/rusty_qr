use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum QrContentType {
    Url,
    Wifi,
    Sms,
    Tel,
    Email,
    Vcard,
    Mecard,
    Geo,
    Gs1,
    TwoDoc,
    Text,
}

impl QrContentType {
    pub const ALL: &'static [QrContentType] = &[
        Self::Url, Self::Wifi, Self::Sms, Self::Tel, Self::Email,
        Self::Vcard, Self::Mecard, Self::Geo, Self::Gs1, Self::TwoDoc, Self::Text,
    ];

    /// Translation key of the type name (see `lang/*.toml`, section `qr_type`).
    pub fn label_key(&self) -> &'static str {
        match self {
            Self::Url    => "qr_type.url",
            Self::Wifi   => "qr_type.wifi",
            Self::Sms    => "qr_type.sms",
            Self::Tel    => "qr_type.tel",
            Self::Email  => "qr_type.email",
            Self::Vcard  => "qr_type.vcard",
            Self::Mecard => "qr_type.mecard",
            Self::Geo    => "qr_type.geo",
            Self::Gs1    => "qr_type.gs1",
            Self::TwoDoc => "qr_type.twodoc",
            Self::Text   => "qr_type.text",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum WifiSecurity { Wpa, Wep, None }

impl WifiSecurity {
    /// Translation key of the security mode (section `wifi_security`).
    pub fn label_key(&self) -> &'static str {
        match self { Self::Wpa => "wifi_security.wpa", Self::Wep => "wifi_security.wep", Self::None => "wifi_security.none" }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum EcLevel { L, M, Q, H }

impl EcLevel {
    pub fn label(&self) -> &'static str {
        match self { Self::L => "L — 7%", Self::M => "M — 15%", Self::Q => "Q — 25%", Self::H => "H — 30%" }
    }
    pub fn to_qrcode(&self) -> qrcode::EcLevel {
        match self {
            Self::L => qrcode::EcLevel::L,
            Self::M => qrcode::EcLevel::M,
            Self::Q => qrcode::EcLevel::Q,
            Self::H => qrcode::EcLevel::H,
        }
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct QrForm {
    pub content_type: QrContentType,
    pub ec_level: EcLevel,
    pub use_micro_qr: bool,

    // URL / Text
    pub url: String,
    pub text: String,

    // WiFi
    pub wifi_ssid: String,
    pub wifi_password: String,
    pub wifi_security: WifiSecurity,
    pub wifi_hidden: bool,

    // SMS
    pub sms_number: String,
    pub sms_message: String,

    // TEL
    pub tel_number: String,

    // Email
    pub email_to: String,
    pub email_subject: String,
    pub email_body: String,

    // vCard
    pub vcard_name: String,
    pub vcard_org: String,
    pub vcard_phone: String,
    pub vcard_email: String,
    pub vcard_url: String,
    pub vcard_address: String,
    pub vcard_note: String,

    // MECARD
    pub mecard_name: String,
    pub mecard_phone: String,
    pub mecard_email: String,
    pub mecard_url: String,

    // GEO
    pub geo_lat: String,
    pub geo_lon: String,
    pub geo_alt: String,

    // GS1 / FNC1
    pub gs1_data: String,

    // 2D-Doc
    pub twod_c40: String,
    pub twod_cert_id: String,
}

impl Default for QrForm {
    fn default() -> Self {
        Self {
            content_type: QrContentType::Url,
            ec_level: EcLevel::M,
            use_micro_qr: false,
            url: String::new(),
            text: String::new(),
            wifi_ssid: String::new(),
            wifi_password: String::new(),
            wifi_security: WifiSecurity::Wpa,
            wifi_hidden: false,
            sms_number: String::new(),
            sms_message: String::new(),
            tel_number: String::new(),
            email_to: String::new(),
            email_subject: String::new(),
            email_body: String::new(),
            vcard_name: String::new(),
            vcard_org: String::new(),
            vcard_phone: String::new(),
            vcard_email: String::new(),
            vcard_url: String::new(),
            vcard_address: String::new(),
            vcard_note: String::new(),
            mecard_name: String::new(),
            mecard_phone: String::new(),
            mecard_email: String::new(),
            mecard_url: String::new(),
            geo_lat: String::new(),
            geo_lon: String::new(),
            geo_alt: String::new(),
            gs1_data: String::new(),
            twod_c40: String::new(),
            twod_cert_id: String::new(),
        }
    }
}

impl QrForm {
    pub fn to_qr_string(&self) -> String {
        match self.content_type {
            QrContentType::Url  => self.url.clone(),
            QrContentType::Text => self.text.clone(),

            QrContentType::Wifi => {
                let sec = match self.wifi_security {
                    WifiSecurity::Wpa  => "WPA",
                    WifiSecurity::Wep  => "WEP",
                    WifiSecurity::None => "nopass",
                };
                let hidden = if self.wifi_hidden { "true" } else { "false" };
                format!("WIFI:T:{};S:{};P:{};H:{};;", sec, self.wifi_ssid, self.wifi_password, hidden)
            }

            QrContentType::Sms => {
                format!("SMSTO:{}:{}", self.sms_number, self.sms_message)
            }

            QrContentType::Tel => {
                format!("tel:{}", self.tel_number)
            }

            QrContentType::Email => {
                let mut s = format!("mailto:{}", self.email_to);
                let mut sep = '?';
                if !self.email_subject.is_empty() {
                    s.push(sep);
                    s.push_str(&format!("subject={}", urlencoded(&self.email_subject)));
                    sep = '&';
                }
                if !self.email_body.is_empty() {
                    s.push(sep);
                    s.push_str(&format!("body={}", urlencoded(&self.email_body)));
                }
                s
            }

            QrContentType::Vcard => {
                let mut v = String::from("BEGIN:VCARD\nVERSION:3.0\n");
                if !self.vcard_name.is_empty()    { v.push_str(&format!("FN:{}\n", self.vcard_name)); }
                if !self.vcard_org.is_empty()     { v.push_str(&format!("ORG:{}\n", self.vcard_org)); }
                if !self.vcard_phone.is_empty()   { v.push_str(&format!("TEL:{}\n", self.vcard_phone)); }
                if !self.vcard_email.is_empty()   { v.push_str(&format!("EMAIL:{}\n", self.vcard_email)); }
                if !self.vcard_url.is_empty()     { v.push_str(&format!("URL:{}\n", self.vcard_url)); }
                if !self.vcard_address.is_empty() { v.push_str(&format!("ADR:;;{};;;;;\n", self.vcard_address)); }
                if !self.vcard_note.is_empty()    { v.push_str(&format!("NOTE:{}\n", self.vcard_note)); }
                v.push_str("END:VCARD");
                v
            }

            QrContentType::Mecard => {
                let mut m = format!("MECARD:N:{};", self.mecard_name);
                if !self.mecard_phone.is_empty() { m.push_str(&format!("TEL:{};", self.mecard_phone)); }
                if !self.mecard_email.is_empty() { m.push_str(&format!("EMAIL:{};", self.mecard_email)); }
                if !self.mecard_url.is_empty()   { m.push_str(&format!("URL:{};", self.mecard_url)); }
                m.push(';');
                m
            }

            QrContentType::Geo => {
                if self.geo_alt.is_empty() {
                    format!("geo:{},{}", self.geo_lat, self.geo_lon)
                } else {
                    format!("geo:{},{},{}", self.geo_lat, self.geo_lon, self.geo_alt)
                }
            }

            QrContentType::Gs1 => {
                // GS1 QR: prepend FNC1 character (0x1D) for GS1-compliant scanners
                format!("\x1d{}", self.gs1_data)
            }

            QrContentType::TwoDoc => {
                // 2D-Doc: DC prefix + version + cert_id + data
                // Minimal format without cryptographic signature (informational only)
                format!("DC04{}{}", self.twod_cert_id, self.twod_c40)
            }
        }
    }

    /// Human-readable hint for the save-name dialog, WITHOUT sensitive fields (no passwords).
    /// Suggested library entry name, in the user language.
    pub fn display_name_hint(&self, lang: &crate::lang::Lang) -> String {
        fn trunc(s: &str, n: usize) -> &str {
            let end = s.char_indices().nth(n).map(|(i,_)| i).unwrap_or(s.len());
            &s[..end]
        }
        match self.content_type {
            QrContentType::Url    => format!("URL \u{2014} {}", trunc(&self.url, 50)),
            QrContentType::Text   => format!("{} \u{2014} {}", lang.t("qr_short.text"), trunc(&self.text, 40)),
            QrContentType::Wifi   => format!("WiFi \u{2014} {}", self.wifi_ssid), // mot de passe exclu
            QrContentType::Sms    => format!("SMS \u{2014} {} \u{2014} {}", self.sms_number, trunc(&self.sms_message, 20)),
            QrContentType::Tel    => format!("{} \u{2014} {}", lang.t("qr_short.tel"), self.tel_number),
            QrContentType::Email  => format!("Email \u{2014} {}", self.email_to),
            QrContentType::Vcard  => format!("vCard \u{2014} {}", self.vcard_name),
            QrContentType::Mecard => format!("MECARD \u{2014} {}", self.mecard_name),
            QrContentType::Geo    => format!("{} \u{2014} {}, {}", lang.t("qr_short.geo"), self.geo_lat, self.geo_lon),
            QrContentType::Gs1    => format!("GS1 \u{2014} {}", trunc(&self.gs1_data, 30)),
            QrContentType::TwoDoc => format!("2D-Doc \u{2014} {}", self.twod_cert_id),
        }
    }
}

fn urlencoded(s: &str) -> String {
    s.chars().flat_map(|c| {
        match c {
            ' '  => "%20".chars().collect::<Vec<_>>(),
            '\n' => "%0A".chars().collect::<Vec<_>>(),
            _ => vec![c],
        }
    }).collect()
}
