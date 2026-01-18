import * as core from "zako:core";
import {project} from "zako:package";

core.info("Hello World!");
core.info(`Project group is ${project.group}`);
core.info(`Project artifact is ${project.artifact}`);
core.info(`Project version is ${project.version}`);
