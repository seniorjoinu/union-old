import { Actor, HttpAgent } from '@dfinity/agent';
import { idlFactory as communitea_idl, canisterId as communitea_id } from 'dfx-generated/communitea';

const agent = new HttpAgent();
const communitea = Actor.createActor(communitea_idl, { agent, canisterId: communitea_id });

document.getElementById("clickMeBtn").addEventListener("click", async () => {
  const name = document.getElementById("name").value.toString();
  const greeting = await communitea.greet(name);

  document.getElementById("greeting").innerText = greeting;
});
