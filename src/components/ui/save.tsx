import {useTranslation} from "react-i18next";


export default function Save() {
    const { t } = useTranslation("config");

    return (
        <button
                className="p-2 px-6 transition duration-200 hover:brightness-75 bg-gradient-to-b from-green-400 to-primary rounded-lg text-black">{t("saveLabel")}
        </button>
    );
}