// src/i18n.js
import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';
import HttpBackend from 'i18next-http-backend';
import LanguageDetector from 'i18next-browser-languagedetector';

export const supportedLngs = {
  en: "English",
    de: "German",
};

i18n
    .use(HttpBackend) // Load translation using http
    .use(LanguageDetector) // Detect language
    .use(initReactI18next) // Passes i18n down to react-i18next
    .init({
        fallbackLng: 'en',
        supportedLngs: Object.keys(supportedLngs),
        debug: true,
        backend: {
            loadPath: '/locales/{{lng}}/{{ns}}.json' // Path to load the translation files
        },
        ns: ['common', 'home', 'about'], // Namespaces used in your project
        defaultNS: 'common', // Default namespace
        interpolation: {
            escapeValue: false // React already escapes values
        }
    });

i18n.on('languageChanged', (lng) => {
    console.log('Language changed to:', lng);
});

export default i18n;
