import {Dispatch, SetStateAction, useState} from "react";
import {invoke} from "@tauri-apps/api/core";
import {
    Dialog, DialogClose,
    DialogContent,
    DialogDescription, DialogFooter,
    DialogHeader,
    DialogTitle,
    DialogTrigger
} from "@components/ui/dialog.tsx";
import {ModInfos} from "@models/mods.ts";
import {useModsState} from "@components/ModsProvider.tsx";
import {useTranslation} from "react-i18next";


export default function UninstallAll({mods, names}: {mods: ModInfos[] | undefined, names: string[]}) {
    const { reloadKey } = useModsState();
    const { t, i18n } = useTranslation('home');

    async function uninstallMods() {
        console.log(mods)
        let newMods: string[] = [];
        if (mods) {
            //Remove names from mods through mods.name
            newMods = mods.filter(mod => names.includes(mod.name)).map(mod => mod.name);
        }
        await invoke("uninstall_mods", {mods: newMods});
        reloadKey[1](prevKey => prevKey + 1);
    }

    return (
        <>
            {names.length > 0 ? (
                <Dialog>
                    <DialogTrigger className="w-full">
                        <button className="w-full relative flex cursor-default select-none items-center rounded-sm px-2 py-1.5 text-sm
                                outline-none hover:bg-accent hover:text-accent-foreground data-[disabled]:pointer-events-none
                                data-[disabled]:opacity-50">{t("uninstallAll")}
                        </button>
                    </DialogTrigger>
                    <DialogContent>
                        <DialogHeader>
                            <DialogTitle className="mb-4">{t("uninstallConfirmation")}</DialogTitle>
                            <DialogDescription>
                                {t("uninstallAllModsDesc")}
                            </DialogDescription>
                        </DialogHeader>
                        <DialogFooter className="sm:justify-end">
                            <DialogClose asChild>
                                <button onClick={uninstallMods}
                                        className="transition duration-300 text-foreground bg-destructive hover:brightness-75 p-2 px-4 rounded">
                                    {t("uninstallAll")}
                                </button>
                            </DialogClose>
                        </DialogFooter>
                    </DialogContent>
                </Dialog>
            ) : (
                <button className="w-full relative flex cursor-default select-none items-center rounded-sm px-2 py-1.5 text-sm
                                outline-none hover:bg-accent hover:text-accent-foreground data-[disabled]:pointer-events-none
                                data-[disabled]:opacity-50 brightness-50">{t("uninstallAll")}
                </button>
            )}
        </>
    )
}