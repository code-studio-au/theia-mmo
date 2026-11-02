// Theia engine-independent prototype. NOT production networking or authoritative server code.
// Node.js CommonJS, no dependencies. Input definitions from Human_*_Quest_Data_Draft.json.
"use strict";
function assertDefinition(def) {
  if (!def || !def.id || !def.completionFlag || !Array.isArray(def.stages)) throw Error("invalid quest");
  if (new Set(def.stages.map(s => s.id)).size !== def.stages.length) throw Error("duplicate stage");
}
function makeCharacter(level=1) { return {level, completed:new Set(), active:new Map(), rewarded:new Set(), worldFlags:new Set()}; }
function availability(def, character) {
  assertDefinition(def);
  if (character.completed.has(def.completionFlag)) return "COMPLETED";
  if (character.active.has(def.id)) return character.active.get(def.id).size === def.stages.filter(s=>s.required).length ? "READY_TO_TURN_IN" : "ACTIVE";
  return character.level >= def.minLevel && def.prerequisites.every(f=>character.completed.has(f)) ? "AVAILABLE" : "LOCKED";
}
function accept(def, character) {
  if (availability(def,character)!=="AVAILABLE") return false;
  character.active.set(def.id,new Set()); return true;
}
function credit(def,character,event) {
  if (!character.active.has(def.id)) return false;
  const credits=character.active.get(def.id);
  const next=def.stages.find(s=>s.required&&!credits.has(s.id));
  if (!next || event.kind!==next.kind || event.targetKey!==next.targetKey) return false;
  credits.add(next.id); return true;
}
function encounterFailed(def,character) {
  const credits=character.active.get(def.id); if (!credits) return;
  for(const stage of def.stages) if(stage.resetPolicy==="encounter_retry") credits.delete(stage.id);
}
function turnIn(def,character,authorizedContact) {
  if (!authorizedContact || availability(def,character)!=="READY_TO_TURN_IN") return false;
  character.active.delete(def.id); character.completed.add(def.completionFlag);
  character.rewarded.add(def.completionFlag); // real rewards require atomic server transaction
  return true;
}
function enemyStep(enemy,ctx) {
  const e={...enemy};
  if(e.state==="DEAD") return e;
  if(e.state==="EVADE" || e.state==="RETURN_HOME") {
    if(ctx.atHome){e.state="IDLE";e.threat={};e.health=e.maxHealth;}
    else e.state="RETURN_HOME";
    return e;
  }
  if(e.state==="IDLE" || e.state==="PATROL") {
    if(ctx.detected && ctx.lineOfSight){e.state="CHASE";e.target=ctx.targetId;}
    return e;
  }
  if(ctx.leashExceeded || !ctx.targetValid){e.state="EVADE";e.target=null;return e;}
  if(!ctx.inRange || !ctx.lineOfSight){e.state="CHASE";return e;}
  e.state="ATTACK";
  e.canAttack=Boolean(ctx.facing && ctx.cooldownReady);
  return e;
}
module.exports={makeCharacter,availability,accept,credit,encounterFailed,turnIn,enemyStep};
