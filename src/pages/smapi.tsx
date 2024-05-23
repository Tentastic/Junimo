import {invoke} from "@tauri-apps/api/core";
import {useEffect, useState} from "react";
import {listen} from "@tauri-apps/api/event";
import SmapiIcon from "../assets/Smapi.png";
import {MoveRight} from "lucide-react";
import SmapiProcess from "@models/smapiProcess";
import {clsx} from "clsx";
import JunimoDance from "@assets/JunimoDance.gif";
import {useTranslation} from "react-i18next";

function InfoText(returnLength: number) {
    const { t } = useTranslation('smapi');

    switch (returnLength) {
        case 0:
            return t("noSmapi");
        case 1:
            return t("firstInstall");
        case 2:
            return t("newVersion");
        case 3:
            return t("newVersion");
        case 4:
            return t("currentVersion");
        default:
            return t("newVersion");
    }
}

export default function SmapiPage() {
    const [installing, setInstalling] = useState<boolean>(false);
    const [downloadLink, setDownloadLink] = useState<string>("");
    const [version, setVersion] = useState<string>("");
    const [oldVersion, setOldVersion] = useState<string>("");
    const [returnLength, setReturnLength] = useState<number>(0);

    const { t } = useTranslation('smapi');

    async function download() {
        if (returnLength < 1 || installing || downloadLink === "" || version === oldVersion) {
            return;
        }
        const path = await invoke<string>('download_smapi', { link: downloadLink });
        console.log(path);
    }

    async function loadSmapi() {
        const smapi = await invoke<string[]>('check_smapi_version');
        console.log(smapi);
        setReturnLength(smapi.length);
        if (smapi.length >= 1) {
            setDownloadLink(smapi[0]);
        }
        if (smapi.length >= 2) {
            setVersion(smapi[1]);
        }
        if (smapi.length >= 3) {
            setOldVersion(smapi[2]);
        }
    }

    useEffect(() => {
        loadSmapi();

        let unsubscribeSmapi = listen('smapi_progress', async (event) => {
            const process = event.payload as SmapiProcess;
            console.log(process);
        });

        return () => {
            unsubscribeSmapi.then((unsub) => unsub());
        };
    }, []);

    return (
        <div className="h-[100vh] w-full pt-5">
            <div className="flex items-center justify-center w-full gap-4">
                <img alt={"Smapi Icon"} src={SmapiIcon} className="w-20 h-20"/>
                <h1 className="text-5xl font-bold">{t("smapiTitle")}</h1>
            </div>
            <p className="w-full text-center text-2xl text-input px-16 mt-10 mb-4">{InfoText((oldVersion !== version) ? returnLength : 4)}</p>

            {(returnLength >= 2 && version !== oldVersion) && (
                <div className="w-full flex justify-center gap-4 text-4xl items-center">
                    <p>{oldVersion}</p>
                    <MoveRight/>
                    <p className="text-green-500">{version}</p>
                </div>
            )}
            <div className="mt-auto w-full flex justify-end items-end absolute bottom-5 right-5">
                <button onClick={download}
                        className={clsx(
                            "p-2 px-6 transition duration-150 bg-green-500 hover:bg-green-600 rounded-lg text-white relative",
                            returnLength < 1 || installing || version === oldVersion || downloadLink === "" ? "opacity-50 cursor-not-allowed" : ""
                        )}>
                    {installing && <img src={JunimoDance} alt="Junimo Dance"
                                        className="w-10 h-10 inline-block mr-2 absolute left-1/2 -translate-x-1/2 -top-7"/>}
                    {t("smapiButton")}
                </button>
            </div>
        </div>
    )
}