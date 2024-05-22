import {useEffect, useState} from "react";
import { Input } from "@components/ui/input"
import { Label } from "@components/ui/label"
import { Folder } from 'lucide-react';
import { Config as ConfigModel } from "@models/config";
import {invoke} from "@tauri-apps/api/core";
import {User} from "@models/user";
import NexusMods from "@assets/NexusMods.png";
import {Switch} from "@components/ui/switch.tsx";
import { open } from '@tauri-apps/plugin-dialog';
import GeneralConfig from "@components/config/GeneralConfig.tsx";
import CheckConfig from "@components/config/CheckConfig.tsx";
import {useModsState} from "@components/ModsProvider.tsx";
import NexusConfig from "@components/config/NexusConfig.tsx";
import {clsx} from "clsx";
import {useTranslation} from "react-i18next";

function PageLoad({page}: {page: number}) {
    switch (page) {
        case 0:
            return (
                <GeneralConfig />
            );
        case 1:
            return (
                <CheckConfig />
            );
        case 2:
            return (
              <NexusConfig />
            );
        default:
            return (
                <GeneralConfig />
            );
    }
}

export default function Config() {
    const [page, setPage] = useState(0);
    const { t } = useTranslation("config");

    return (
        <div className="w-full h-[100vh] p-6 pl-6 pt-4 flex flex-col justify-start items-start">
            <div className="flex gap-2 mb-4">
                <div className={clsx("p-1 px-3 rounded bg-muted cursor-pointer", page == 0 && "bg-primary text-black")} onClick={x => setPage(0)}>{t("generalLabel")}</div>
                <div className={clsx("p-1 px-3 rounded bg-muted cursor-pointer", page == 1 && "bg-primary text-black")} onClick={x => setPage(1)}>{t("checkLabel")}</div>
                <div className={clsx("p-1 px-3 rounded bg-muted cursor-pointer", page == 2 && "bg-primary text-black")} onClick={x => setPage(2)}>NexusMods</div>
            </div>
            <PageLoad page={page}/>
        </div>
    )
}