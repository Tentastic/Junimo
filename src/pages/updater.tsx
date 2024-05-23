import Logo from "@assets/Logo.webp";
import {MoveRight} from "lucide-react";
import {clsx} from "clsx";
import JunimoDance from "@assets/JunimoDance.gif";
import {useTranslation} from "react-i18next";
import {check, Update} from "@tauri-apps/plugin-updater";
import {relaunch} from "@tauri-apps/plugin-process";
import {useEffect, useState} from "react";


export default function Updater() {
    const [installing, setInstalling] = useState<boolean>(false);
    const [update, setUpdate] = useState<Update | null>(null);

    const { t } = useTranslation('updater');

    async function checkUpdate() {
        const updateChecked = await check();
        console.log(updateChecked)
        if (updateChecked?.available) {
            setUpdate(updateChecked);
            console.log(`Update to ${updateChecked.version} available! Date: ${updateChecked.date}`);
            console.log(`Release notes: ${updateChecked.body}`);

        }
    }

    async function install() {
        if (update === null || installing) {
            return;
        }
        setInstalling(true);
        await update.downloadAndInstall();
        await relaunch();
    }

    useEffect(() => {
        checkUpdate();
    }, []);

    return (
        <div className="h-[100vh] w-full pt-5">
            <div className="flex items-center justify-center w-full gap-4">
                <img alt={"Smapi Icon"} src={Logo} className="w-20 h-20"/>
                <h1 className="text-5xl font-bold">{t("updaterTitle")}</h1>
            </div>
            {update !== null ? (
                <p className="w-full text-center text-2xl text-input px-16 mt-10 mb-4">{t("updaterUpdateAvailable")}</p>
            ) : (
                <p className="w-full text-center text-2xl text-input px-16 mt-10 mb-4">{t("downloaderUpToDate")}</p>
            )}

            <div className="mt-auto w-full flex justify-end items-end absolute bottom-5 right-5">
            <button onClick={install}
                        className={clsx(
                            "p-2 px-6 transition duration-150 bg-green-500 hover:bg-green-600 rounded-lg text-white relative",
                            update === null || installing ? "opacity-50 cursor-not-allowed" : ""
                        )}>
                    {installing && <img src={JunimoDance} alt="Junimo Dance"
                                        className="w-10 h-10 inline-block mr-2 absolute left-1/2 -translate-x-1/2 -top-7"/>}
                    {t("updaterButton")}
                </button>
            </div>
        </div>
    )
}