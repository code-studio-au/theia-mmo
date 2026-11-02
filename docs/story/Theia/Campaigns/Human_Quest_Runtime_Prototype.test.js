// Run with: node Human_Quest_Runtime_Prototype.test.js
"use strict";
const assert=require("node:assert/strict");
const {makeCharacter,availability,accept,credit,encounterFailed,turnIn,enemyStep}=require("./Human_Quest_Runtime_Prototype.js");
const q1={id:"HUMAN_Q001",completionFlag:"HUMAN_Q001_DONE",minLevel:1,prerequisites:[],stages:[
{id:"S01",kind:"talk",targetKey:"darian",required:true,resetPolicy:"persist_on_logout"},
{id:"S02",kind:"defend",targetKey:"worker",required:true,resetPolicy:"encounter_retry"},
{id:"S03",kind:"talk",targetKey:"elara",required:true,resetPolicy:"persist_on_logout"}]};
const q2={id:"HUMAN_Q002",completionFlag:"HUMAN_Q002_DONE",minLevel:5,prerequisites:["HUMAN_Q001_DONE"],stages:[{id:"S01",kind:"inspect",targetKey:"record",required:true,resetPolicy:"persist_on_logout"}]};
const c=makeCharacter();
assert.equal(availability(q2,c),"LOCKED");
assert.equal(availability(q1,c),"AVAILABLE");
assert.equal(accept(q1,c),true);
assert.equal(credit(q1,c,{kind:"talk",targetKey:"elara"}),false); // ordered
assert.equal(credit(q1,c,{kind:"talk",targetKey:"darian"}),true);
assert.equal(credit(q1,c,{kind:"talk",targetKey:"darian"}),false); // no duplicate
assert.equal(credit(q1,c,{kind:"defend",targetKey:"worker"}),true);
encounterFailed(q1,c);
assert.equal(c.active.get(q1.id).has("S01"),true);
assert.equal(c.active.get(q1.id).has("S02"),false);
assert.equal(credit(q1,c,{kind:"defend",targetKey:"worker"}),true);
assert.equal(credit(q1,c,{kind:"talk",targetKey:"elara"}),true);
assert.equal(availability(q1,c),"READY_TO_TURN_IN");
assert.equal(turnIn(q1,c,false),false);
assert.equal(turnIn(q1,c,true),true);
assert.equal(turnIn(q1,c,true),false); // exactly once
assert.equal(availability(q2,c),"LOCKED");
c.level=5;assert.equal(availability(q2,c),"AVAILABLE");
let e={state:"IDLE",maxHealth:100,health:100,threat:{}};
e=enemyStep(e,{detected:true,lineOfSight:true,targetId:"p1"});assert.equal(e.state,"CHASE");
e=enemyStep(e,{targetValid:true,inRange:true,lineOfSight:true,facing:false,cooldownReady:true});assert.equal(e.canAttack,false);
e=enemyStep(e,{targetValid:true,inRange:true,lineOfSight:true,facing:true,cooldownReady:true});assert.equal(e.canAttack,true);
e=enemyStep(e,{leashExceeded:true,targetValid:true});assert.equal(e.state,"EVADE");
e=enemyStep(e,{atHome:false});assert.equal(e.state,"RETURN_HOME");
e=enemyStep(e,{atHome:true});assert.equal(e.state,"IDLE");
console.log("Quest lifecycle, retry, level gates and enemy leash/facing checks passed.");
