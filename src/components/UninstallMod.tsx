import {invoke} from "@tauri-apps/api/core";
import {ModInfos} from "@models/mods.ts";
import {Dispatch, SetStateAction} from "react";
import {
    Dialog,
    DialogClose,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
    DialogTrigger
} from "@components/ui/dialog.tsx";
import {useModsState} from "@components/ModsProvider.tsx";
import {useTranslation} from "react-i18next";


export default function UninstallMod({name}: {name: string}) {
    const { reloadKey } = useModsState();
    const { t, i18n } = useTranslation('home');

    async function uninstallMods() {
        await invoke<ModInfos[]>('uninstall_mod', {name: name});
        reloadKey[1](prevKey => prevKey + 1);
    }

    return (
        <Dialog>
            <DialogTrigger className="w-full">
                <button className="w-full relative flex cursor-default select-none items-center rounded-sm px-2 py-1.5 text-sm
                                outline-none hover:bg-accent hover:text-accent-foreground data-[disabled]:pointer-events-none
                                data-[disabled]:opacity-50">
                    {t("uninstallLabel")}
                </button>
            </DialogTrigger>
            <DialogContent>
                <DialogHeader>
                    <DialogTitle className="mb-4">{t("uninstallConfirmation")}</DialogTitle>
                    <DialogDescription>
                        {t("uninstallModsDesc")}
                    </DialogDescription>
                </DialogHeader>
                <DialogFooter className="sm:justify-end">
                    <DialogClose asChild>
                        <button onClick={uninstallMods}
                                className="transition duration-300 text-foreground bg-destructive hover:brightness-75 p-2 px-4 rounded">
                            {t("uninstallLabel")}
                        </button>
                    </DialogClose>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    )
}