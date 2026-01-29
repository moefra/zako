import * as core from "zako:core";
import {project} from "zako:package";

core.info("Hello World!");
core.trace("This is a trace message");
core.debug("This is a debug message");
core.info("This is a info message");
core.warn("This is a warn message");
core.error("This is a error message");
core.info(`Project group is ${project.group}`);
core.info(`Project artifact is ${project.artifact}`);
core.info(`Project version is ${project.version}`);
