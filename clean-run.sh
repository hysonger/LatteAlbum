# 清理并重新运行项目

rm cache/*
rm database.db
mvn clean package spring-boot:run -DskipTests
