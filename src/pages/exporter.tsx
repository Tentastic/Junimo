import {useEffect, useState} from "react";
import {Profile} from "@models/profile.ts";
import {invoke} from "@tauri-apps/api/core";
import {Select, SelectContent, SelectItem, SelectTrigger, SelectValue} from "@components/ui/select.tsx";
import {Label} from "@components/ui/label.tsx";
import {Input} from "@components/ui/input.tsx";
import {Folder} from "lucide-react";
import {clsx} from "clsx";
import JunimoDance from "../assets/JunimoDance.gif";
import {open} from "@tauri-apps/plugin-dialog";
import {useTranslation} from "react-i18next";

export default function Exporter() {
    const [profiles, setProfiles] = useState<Profile[]>([]);
    const [exportPath, setExportPath] = useState("");
    const [selectedProfile, setSelectedProfile] = useState<string>("");
    const [valid, setValid] = useState(false);
    const [exporting, setExporting] = useState(false);

    const { t } = useTranslation('export');

    function newSelected(e: string) {
        setSelectedProfile(e);
        if (e !== "" && exportPath !== "") {
            setValid(true);
        }
        else {
            setValid(false);
        }
    }

    async function fetchPath() {
        const path = await open({
            multiple: false,
            directory: true,
        });
        if (path !== null) {
            setExportPath(path);
        }

        if (selectedProfile !== "" && path !== "") {
            setValid(true);
        }
        else {
            setValid(false);
        }
    }

    async function exportProfile() {
        if (selectedProfile === "" || exportPath === "") {
            return;
        }

        setExporting(true);
        await invoke<boolean>('export_profile', {name: selectedProfile, path: exportPath});
    }

    async function loadProfile() {
        const profilePath = await invoke<string>('profile_path');
        const loadedProfiles = await invoke<Profile[]>('get_profiles', {path: profilePath});
        setProfiles(loadedProfiles);
    }

    useEffect(() => {
        loadProfile();
    }, []);

    return (
        <div className="w-full h-[100vh] p-6 flex flex-col justify-start items-start">
            <h1 className="text-3xl text-left font-bold mb-4 text-primary">{t("exportTitle")}</h1>
            <div className="w-full">
                <p className="ml-1 mb-1 text-lg text-muted-foreground">{t("selectProfileLabel")}</p>
                <Select onValueChange={e => newSelected(e)}>
                    <SelectTrigger className="w-[180px]">
                        <SelectValue placeholder={t("selectProfilePlaceholer")} />
                    </SelectTrigger>
                    <SelectContent>
                        <SelectItem value="All Profiles">{t("exportAllProfiles")}</SelectItem>
                        {profiles.map((profile, index) => (
                            <SelectItem key={index} value={profile.name}>{profile.name}</SelectItem>
                        ))}
                    </SelectContent>
                </Select>
            </div>
            <div className="flex flex-col mt-4 w-full">
                <Label htmlFor="path" className="ml-1 mb-1 text-lg text-muted-foreground">{t("exportPathLabel")}</Label>
                <div className="flex gap-2">
                    <Input id="path" placeholder={t("exportPathPlaceholder")}
                           value={exportPath} onChange={x => setExportPath(x.target.value)}/>
                    <button onClick={fetchPath} className="w-10 h-10 flex items-center justify-center transition duration-150 border rounded-lg
                        bg-muted hover:bg-muted-dark">
                        <Folder size={20}/>
                    </button>
                </div>
            </div>
            <div className="mt-auto w-full flex justify-end items-end">
                <button onClick={exportProfile}
                        className={clsx(
                            "p-2 px-6 transition duration-150 bg-green-500 hover:bg-green-600 rounded-lg text-white relative",
                            !valid || exporting ? "opacity-50 cursor-not-allowed" : ""
                        )}>
                    {exporting && <img src={JunimoDance} alt="Junimo Dance" className="w-10 h-10 inline-block mr-2 absolute left-1/2 -translate-x-1/2 -top-7" /> }
                    {t("exportTitle")}
                </button>
            </div>
        </div>
    )
}