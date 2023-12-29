import { createSignal } from "solid-js";
import logo from "./assets/logo.svg";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

function App() {
  const [greetMsg, setGreetMsg] = createSignal("");
  const [name, setName] = createSignal("");

  async function greet() {
    // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
    setGreetMsg(await invoke("greet", { name: name() }));
  }

  const [img, setImg] = createSignal("")

  return (
    <div class="container">
      <h1>Welcome to Tauri!</h1>

      <button onClick={() => {
        invoke('update_screen').then((res: any) => {
          console.log("updated screen: ", res)
          const blob = new Blob([new Uint8Array(res)], { type: 'image/png' });
          setImg(URL.createObjectURL(blob))
        }).catch((err) => {
          console.log("fialed to update screen: ", err)
        })
      }}>update_screen</button>

      <img src={img()}/>
    </div>
  );
}

export default App;
