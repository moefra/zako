import * as core from "zako:core";
import * as project from "zako:project";

core.trace(`Benchmark version is ${project.config.version}`);

core.requireZakoVersion(">=1.0.0 && <3.0.0");

if(project.config.log){
    project.builds.push("./logging");
}

export default project;
