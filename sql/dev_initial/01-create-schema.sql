---- Base app schema


-- User
CREATE TABLE "user" (
  id BIGINT GENERATED BY DEFAULT AS IDENTITY (START WITH 1000) PRIMARY KEY,

  username varchar(128) NOT NULL UNIQUE, 

  -- Auth
  pwd varchar(256),
  pwd_salt uuid NOT NULL DEFAULT gen_random_uuid(),
  token_salt uuid NOT NULL DEFAULT gen_random_uuid(),

  -- Timestamps
  cid bigint NOT NULL,
  ctime timestamp with time zone NOT NULL,
  mid bigint NOT NULL,
  mtime timestamp with time zone NOT NULL  
);

-- Course
CREATE TYPE course_state AS ENUM ('Draft', 'Published', 'Archived');

CREATE TABLE course (
  id BIGINT GENERATED BY DEFAULT AS IDENTITY (START WITH 1000) PRIMARY KEY,
  title varchar(256) NOT NULL UNIQUE,
  description varchar(256) NOT NULL,
  course_type varchar(256) NOT NULL,
  price float4 NOT NULL,
  color varchar(256) NOT NULL,
  published_date timestamp with time zone,
  image_url varchar(256),
  state course_state NOT NULL default 'Draft',

  -- Timestamps
  cid bigint NOT NULL,
  ctime timestamp with time zone NOT NULL,
  mid bigint NOT NULL,
  mtime timestamp with time zone NOT NULL  
);

-- User in course
CREATE TYPE user_course_roles AS ENUM ('Student', 'Creator');

CREATE TABLE users_courses (
  user_id BIGINT NOT NULL,
  course_id BIGINT NOT NULL,
  user_role user_course_roles NOT NULL default 'Student',

  PRIMARY KEY (user_id, course_id),
  CONSTRAINT fk_user FOREIGN KEY (user_id) REFERENCES "user"(id) ON DELETE CASCADE,
  CONSTRAINT fk_course FOREIGN KEY (course_id) REFERENCES course(id) ON DELETE CASCADE
);

-- Lesson
CREATE TABLE lesson (
  id BIGINT GENERATED BY DEFAULT AS IDENTITY (START WITH 1000) PRIMARY KEY,
  course_id BIGINT NOT NULL,

  title varchar(256) NOT NULL,
  lesson_order integer NOT NULL
);

-- Exercise 
CREATE TABLE exercise (
  id BIGINT GENERATED BY DEFAULT AS IDENTITY (START WITH 1000) PRIMARY KEY,
  lesson_id BIGINT NOT NULL,

  title varchar(256) NOT NULL,
  type varchar(256) NOT NULL,
  body jsonb NOT NULL,
  has_free_preview boolean NOT NULL,
  exercise_order integer NOT NULL,

  CONSTRAINT fk_lesson FOREIGN KEY (lesson_id) REFERENCES lesson(id) ON DELETE CASCADE
);

-- Exercises completed 
CREATE TABLE exercise_completed (
  user_id BIGINT NOT NULL,
  exercise_id BIGINT NOT NULL,
  points integer NOT NULL,

  PRIMARY KEY (user_id, exercise_id),
  CONSTRAINT fk_user FOREIGN KEY (user_id) REFERENCES "user"(id),
  CONSTRAINT fk_exercise FOREIGN KEY (exercise_id) REFERENCES exercise(id)
);