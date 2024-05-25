---- Base app schema


-- User
CREATE TABLE "user" (
  id BIGINT GENERATED BY DEFAULT AS IDENTITY (START WITH 1000) PRIMARY KEY,

  username varchar(128) NOT NULL UNIQUE, 

  -- Auth
  pwd varchar(256),
  pwd_salt uuid NOT NULL,
  token_salt uuid NOT NULL,

  -- Timestamps
  cid bigint NOT NULL,
  ctime timestamp with time zone NOT NULL,
  mid bigint NOT NULL,
  mtime timestamp with time zone NOT NULL  
);

-- Course
-- CREATE TYPE course_state AS ENUM ('Draft', 'Published', 'Archived');

CREATE TABLE course (
  id BIGINT GENERATED BY DEFAULT AS IDENTITY (START WITH 1000) PRIMARY KEY,
  title varchar(256) NOT NULL UNIQUE,
  description varchar(256) NOT NULL,
  course_type varchar(256) NOT NULL,
  price float8 NOT NULL,
  color varchar(256) NOT NULL,
  published_date timestamp with time zone,
  img_url varchar(256),
  state varchar(256) NOT NULL default 'Draft',

  -- Timestamps
  cid bigint NOT NULL,
  ctime timestamp with time zone NOT NULL,
  mid bigint NOT NULL,
  mtime timestamp with time zone NOT NULL  
);

-- User in course
-- CREATE TYPE user_course_roles AS ENUM ('Student', 'Creator');

CREATE TABLE users_courses (
  id BIGINT GENERATED BY DEFAULT AS IDENTITY (START WITH 1000) PRIMARY KEY,
  user_id BIGINT NOT NULL,
  course_id BIGINT NOT NULL,
  user_role varchar(256) NOT NULL default 'Student',

  CONSTRAINT fk_user FOREIGN KEY (user_id) REFERENCES "user"(id) ON DELETE CASCADE,
  CONSTRAINT fk_course FOREIGN KEY (course_id) REFERENCES course(id) ON DELETE CASCADE
);

-- Lesson
CREATE TABLE lesson (
  id BIGINT GENERATED BY DEFAULT AS IDENTITY (START WITH 1000) PRIMARY KEY,
  course_id BIGINT NOT NULL,

  title varchar(256) NOT NULL,
  description varchar(256) NOT NULL,
  lesson_order integer NOT NULL
);

-- Lessons completed 
CREATE TABLE lesson_progress (
  user_course_id BIGINT NOT NULL,
  lesson_id BIGINT NOT NULL,

  date_started timestamp with time zone NOT NULL,
  date_complete timestamp with time zone,  

  state varchar(256) NOT NULL default 'InProgress', 

  PRIMARY KEY (user_course_id, lesson_id),
  CONSTRAINT fk_users_courses FOREIGN KEY (user_course_id) REFERENCES users_courses(id),
  CONSTRAINT fk_lesson FOREIGN KEY (lesson_id) REFERENCES lesson(id)
);

-- Exercise 
CREATE TABLE exercise (
  id BIGINT GENERATED BY DEFAULT AS IDENTITY (START WITH 1000) PRIMARY KEY,
  lesson_id BIGINT NOT NULL,

  title varchar(256) NOT NULL,
  description varchar(256) NOT NULL,
  type varchar(256) NOT NULL,
  exercise_order integer NOT NULL,
  body jsonb NOT NULL,

  difficult varchar(256) NOT NULL default 'Read',
  time_to_complete timestamp with time zone,  

  CONSTRAINT fk_lesson FOREIGN KEY (lesson_id) REFERENCES lesson(id) ON DELETE CASCADE
);

-- Exercises completed 
CREATE TABLE exercise_completion (
  exercise_id BIGINT NOT NULL,
  user_in_course_id BIGINT NOT NULL,

  points_scored integer NOT NULL,
  number_of_attempts integer NOT NULL,

  date_started timestamp with time zone NOT NULL,
  date_completed timestamp with time zone NOT NULL,

  state varchar(256) NOT NULL default 'InProgress', 
  body jsonb NOT NULL,

  PRIMARY KEY (user_in_course_id, exercise_id),
  CONSTRAINT fk_users_courses FOREIGN KEY (user_in_course_id) REFERENCES users_courses(id),
  CONSTRAINT fk_exercise FOREIGN KEY (exercise_id) REFERENCES exercise(id)
);