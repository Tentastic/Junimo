import "../App.css";
import "../styles.css";
import {invoke} from "@tauri-apps/api/tauri";

export default function Test() {
    async function addMod() {
        // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
        await invoke("add_mod")
    }

    return (
        <div>
            <h1 className="text-sm text-red-500">Test Page</h1>
            <button onClick={addMod}>Add Mod</button>
        </div>
    )
}